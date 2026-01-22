use std::{
    borrow::Cow,
    path::Path,
    sync::Arc,
};

use crate::{
    config::ConversionNodeConfig,
    dictionary::{
        DictType,
        Dictionary,
    },
    error::Result,
};

pub struct ConversionChain {
    dictionaries: Vec<Arc<dyn Dictionary>>,
}

// We don't use a segmenter here, see https://github.com/BYVoid/OpenCC/issues/475

impl ConversionChain {
    pub fn from_config(config: &[ConversionNodeConfig], config_dir: &Path) -> Result<Self> {
        let dictionaries = config
            .iter()
            .map(|node| DictType::from_config(&node.dict, config_dir))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { dictionaries })
    }

    pub fn from_config_embedded(config: &[ConversionNodeConfig]) -> Result<Self> {
        let dictionaries = config
            .iter()
            .map(|node| DictType::from_config_embedded(&node.dict))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { dictionaries })
    }

    pub fn convert(&self, text: &str) -> String {
        let mut current_cow = Cow::Borrowed(text);

        for dict in &self.dictionaries {
            current_cow = Self::apply_dict(current_cow, dict.as_ref());
        }

        current_cow.into_owned()
    }

    fn apply_dict<'a>(text: Cow<'a, str>, dict: &dyn Dictionary) -> Cow<'a, str> {
        let mut result: Option<String> = None;
        let mut i = 0;

        while i < text.len() {
            let remaining_text = &text[i..];
            if let Some((key, values)) = dict.match_prefix(remaining_text) {
                if let Some(values_0) = values.first() {
                    let res_str = result.get_or_insert_with(|| {
                        let mut new_string = String::with_capacity(text.len());
                        new_string.push_str(&text[..i]);
                        new_string
                    });

                    res_str.push_str(values_0);
                    i += key.len();
                } else {
                    i = advance_char(i, remaining_text, result.as_mut());
                }
            } else {
                i = advance_char(i, remaining_text, result.as_mut());
            }
        }

        result.map(Cow::Owned).unwrap_or(text)
    }
}

fn advance_char(mut i: usize, remaining_text: &str, result: Option<&mut String>) -> usize {
    if let Some(ch) = remaining_text.chars().next() {
        if let Some(res_str) = result {
            res_str.push(ch);
        }
        i += ch.len_utf8();
    } else {
        i = remaining_text.len() + 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        fmt::Debug,
    };

    use super::*;
    use crate::dictionary::Dictionary;

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
        fn match_prefix<'a, 'b>(&'a self, word: &'b str) -> Option<(&'b str, Vec<String>)> {
            let mut longest_match_len = 0;
            let mut result: Option<(&'b str, Vec<String>)> = None;

            for (key, values) in &self.entries {
                if word.starts_with(key) && key.len() > longest_match_len {
                    longest_match_len = key.len();
                    let string_values = values
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect();
                    result = Some((&word[..key.len()], string_values));
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
