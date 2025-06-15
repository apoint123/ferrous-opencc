//! 提供文本分词功能

use crate::config::SegmentationConfig;
use crate::dictionary::{DictType, Dictionary};
use crate::error::{OpenCCError, Result};
use std::path::Path;
use std::sync::Arc;

/// 定义分词器行为的 trait
pub trait Segmentation: Send + Sync {
    /// 将输入的文本分割成一个字符串切片的向量
    fn segment<'a>(&self, text: &'a str) -> Vec<&'a str>;
}

/// 一个用作工厂的枚举，用于创建不同类型的分词器
pub enum SegmentationType {
    /// 代表最大匹配分词算法
    MaxMatch(Arc<dyn Dictionary>),
}

impl SegmentationType {
    /// 从文件加载配置来创建分词器类型
    pub fn from_config(config: &SegmentationConfig, config_dir: &Path) -> Result<Self> {
        match config.seg_type.as_str() {
            "mm" | "mmseg" => {
                let dict = DictType::from_config(&config.dict, config_dir)?;
                Ok(SegmentationType::MaxMatch(dict))
            }
            _ => Err(OpenCCError::InvalidConfig(format!(
                "Unsupported segmentation type: {}",
                config.seg_type
            ))),
        }
    }

    /// 从嵌入式资源加载配置来创建分词器类型
    #[cfg(feature = "embed-dictionaries")]
    pub fn from_config_embedded(config: &SegmentationConfig) -> Result<Self> {
        match config.seg_type.as_str() {
            "mm" | "mmseg" => {
                let dict = DictType::from_config_embedded(&config.dict)?;
                Ok(SegmentationType::MaxMatch(dict))
            }
            _ => Err(OpenCCError::InvalidConfig(format!(
                "Unsupported segmentation type: {}",
                config.seg_type
            ))),
        }
    }

    /// 消费此 `SegmentationType` 并返回一个实现了 `Segmentation` trait 的具体分词器对象
    pub fn into_segmenter(self) -> Box<dyn Segmentation> {
        match self {
            SegmentationType::MaxMatch(dict) => Box::new(MaxMatchSegmentation::new(dict)),
        }
    }
}

/// 一个使用正向最大匹配算法的分词器。
/// 该算法会贪婪地从词典中查找与剩余文本开头匹配的最长的词。
pub struct MaxMatchSegmentation {
    dict: Arc<dyn Dictionary>,
}

impl MaxMatchSegmentation {
    /// 使用指定的词典创建一个新的 `MaxMatchSegmentation` 实例
    pub fn new(dict: Arc<dyn Dictionary>) -> Self {
        Self { dict }
    }
}

impl Segmentation for MaxMatchSegmentation {
    /// 对给定的文本执行正向最大匹配分词。
    /// 遍历文本，在每个位置找到词典中能作为剩余文本前缀的最长词语。
    /// 如果没有找到匹配，则将当前位置的单个字符作为一个片段。
    fn segment<'a>(&self, text: &'a str) -> Vec<&'a str> {
        let mut segments = Vec::new();
        let mut start = 0;

        while start < text.len() {
            let remaining_text = &text[start..];
            if let Some((matched_key, _)) = self.dict.match_prefix(remaining_text) {
                // 如果在词典中找到匹配，则将匹配到的词作为一个片段
                segments.push(matched_key);
                start += matched_key.len();
            } else {
                // 如果没有找到匹配，则安全地分割出当前位置的第一个字符
                let ch_end = remaining_text
                    .char_indices()
                    .nth(1)
                    .map_or(remaining_text.len(), |(idx, _)| idx);
                segments.push(&remaining_text[..ch_end]);
                start += ch_end;
            }
        }
        segments
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
    fn test_max_match_segmentation() {
        let mut dict = MockDict::default();
        dict.add_entry("一个", "一個");
        dict.add_entry("丑恶", "醜惡");
        dict.add_entry("的", "的");
        dict.add_entry("汉字", "漢字");
        let dict_arc: Arc<dyn Dictionary> = Arc::new(dict);

        let segmenter = MaxMatchSegmentation::new(dict_arc);

        let text1 = "一个丑恶的汉字";
        let segments1 = segmenter.segment(text1);
        assert_eq!(segments1, vec!["一个", "丑恶", "的", "汉字"]);

        let text2 = "一个人的汉字";
        let segments2 = segmenter.segment(text2);
        assert_eq!(segments2, vec!["一个", "人", "的", "汉字"]);

        let mut dict2 = MockDict::default();
        dict2.add_entry("中国", "中國");
        dict2.add_entry("中国人", "中國人");
        let dict2_arc: Arc<dyn Dictionary> = Arc::new(dict2);
        let segmenter2 = MaxMatchSegmentation::new(dict2_arc);

        let text3 = "我是中国人";
        let segments3 = segmenter2.segment(text3);
        assert_eq!(segments3, vec!["我", "是", "中国人"]);

        let text4 = "";
        let segments4 = segmenter.segment(text4);
        assert_eq!(segments4, Vec::<&str>::new());
    }
}
