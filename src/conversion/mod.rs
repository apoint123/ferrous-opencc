//! 负责处理文本转换的核心逻辑

use crate::config::ConversionNodeConfig;
use crate::dictionary::{DictType, Dictionary};
use crate::error::Result;
use std::borrow::Cow;
use std::path::Path;
use std::sync::Arc;

/// 负责执行一个或多个转换步骤
pub struct ConversionChain {
    /// 按顺序应用的词典列表
    dictionaries: Vec<Arc<dyn Dictionary>>,
}

// 为何不使用分词器?
//
// 考虑一个“简体 -> 台湾正体”的转换，其中包含地区用词的替换，例如将“内存”转换为“記憶體”。
// `OpenCC` 的标准流程是：
// 1.  用一个通用词典（如 `STCharacters`）进行初步简繁转换。在这个阶段，“内存”会变成“內存”。
// 2.  用一个台湾地区用语词典（如 `TWPhrasesIT`）进行转换，它包含规则 `內存 -> 記憶體`。
//
// 如果先用 `STPhrases` 进行分词，而这个词典本身不包含“内存”这个词条，那么分词器会将它拆分为 `["内", "存"]`。
// 在后续的转换步骤中，程序将无法看到完整的“內存”这个词组，因此 `TWPhrasesIT` 中的 `內存 -> 記憶體` 规则也就无法被匹配到。

impl ConversionChain {
    /// 从文件加载配置来创建一个新的转换链
    pub(super) fn from_config(config: &[ConversionNodeConfig], config_dir: &Path) -> Result<Self> {
        let dictionaries = config
            .iter()
            .map(|node| DictType::from_config(&node.dict, config_dir))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { dictionaries })
    }

    /// 从嵌入式资源加载配置来创建一个新的转换链
    pub(super) fn from_config_embedded(config: &[ConversionNodeConfig]) -> Result<Self> {
        let dictionaries = config
            .iter()
            // 调用 DictType 即将创建的嵌入式构造函数
            .map(|node| DictType::from_config_embedded(&node.dict))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { dictionaries })
    }

    /// 对分词后的片段执行转换。
    /// 每个文本片段都会经过整个词典转换链的处理。
    pub(super) fn convert(&self, text: &str) -> String {
        let mut current_cow = Cow::Borrowed(text);

        // 将 Cow 传递给转换链中的每个词典
        for dict in &self.dictionaries {
            current_cow = Self::apply_dict(current_cow, dict.as_ref());
        }

        current_cow.into_owned()
    }

    /// 使用单个词典，通过贪婪替换策略对文本进行一次完整的转换
    fn apply_dict<'a>(text: Cow<'a, str>, dict: &dyn Dictionary) -> Cow<'a, str> {
        let mut result: Option<String> = None;
        let mut i = 0;

        while i < text.len() {
            let remaining_text = &text[i..];
            if let Some((key, [values_0, ..])) = dict.match_prefix(remaining_text) {
                // 找到了一个匹配
                let res_str = result.get_or_insert_with(|| {
                    // 第一次进行更改时，分配结果字符串，并复制到已经跳过的原始字符串部分
                    let mut new_string = String::with_capacity(text.len());
                    new_string.push_str(&text[..i]);
                    new_string
                });

                // 追加转换后的值，总是选择第一个候选词
                res_str.push_str(values_0);
                i += key.len();
            } else {
                // 在这个位置没有找到匹配
                if let Some(ch) = remaining_text.chars().next() {
                    if let Some(res_str) = result.as_mut() {
                        // 如果已经在构建一个字符串，追加这个字符
                        res_str.push(ch);
                    }
                    // 如果没有在构建字符串（result 是 None），我们什么也不做
                    // 因为我们仍然有效地“借用”着原始的切片
                    i += ch.len_utf8();
                } else {
                    // 此处理论上不可达，因为有 while i < text.len()
                    break;
                }
            }
        }

        // 如果 `result` 仍然是 `None`，意味着没有进行任何替换
        // 我们可以返回原始的、借用的字符串切片。否则，我们返回新创建的 `String`
        result.map(Cow::Owned).unwrap_or(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::Dictionary;
    use std::collections::HashMap;
    use std::fmt::Debug;

    #[derive(Debug, Default)]
    struct MockDict {
        entries: HashMap<String, Vec<Arc<str>>>,
        max_key_length: usize,
    }

    impl MockDict {
        fn add_entry(&mut self, key: &str, value: &str) {
            self.entries.insert(key.to_string(), vec![Arc::from(value)]);
            self.max_key_length = self.max_key_length.max(key.len());
        }
    }

    impl Dictionary for MockDict {
        fn match_prefix<'a, 'b>(&'a self, word: &'b str) -> Option<(&'b str, &'a [Arc<str>])> {
            let mut longest_match_len = 0;
            let mut result: Option<(&'b str, &'a [Arc<str>])> = None;

            // 测试中就简单实现了
            for (key, values) in &self.entries {
                if word.starts_with(key) && key.len() > longest_match_len {
                    longest_match_len = key.len();
                    result = Some((&word[..key.len()], values.as_slice()));
                }
            }
            result
        }

        fn max_key_length(&self) -> usize {
            self.max_key_length
        }
    }

    #[test]
    fn test_apply_dict_greedy_replacement() {
        let mut dict = MockDict::default();
        dict.add_entry("a", "A");
        dict.add_entry("ab", "AB");
        dict.add_entry("abc", "ABC");
        let dict_arc: Arc<dyn Dictionary> = Arc::new(dict);

        let result = ConversionChain::apply_dict(Cow::Borrowed("abcdef"), dict_arc.as_ref());
        assert_eq!(result, "ABCdef");

        let result2 = ConversionChain::apply_dict(Cow::Borrowed("abac"), dict_arc.as_ref());
        assert_eq!(result2, "ABAc");

        let result3 = ConversionChain::apply_dict(Cow::Borrowed("zyxw"), dict_arc.as_ref());
        assert_eq!(result3, "zyxw");
    }

    #[test]
    fn test_conversion_chain_with_multiple_dicts() {
        // Dict 1: s -> t
        let mut dict1 = MockDict::default();
        dict1.add_entry("一个", "一個");
        dict1.add_entry("项目", "項目");
        let dict1_arc: Arc<dyn Dictionary> = Arc::new(dict1);

        // Dict 2: t -> hk
        let mut dict2 = MockDict::default();
        dict2.add_entry("一個", "一個");
        dict2.add_entry("項目", "專案");
        let dict2_arc: Arc<dyn Dictionary> = Arc::new(dict2);

        let chain = ConversionChain {
            dictionaries: vec![dict1_arc, dict2_arc],
        };

        let text_to_convert = "一个项目";

        let result = chain.convert(text_to_convert);

        // "一个" -> "一個" (dict1) -> "一個" (dict2)
        // "项目" -> "項目" (dict1) -> "專案" (dict2)
        assert_eq!(result, "一個專案");
    }
}
