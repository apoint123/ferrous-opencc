use std::{
    fmt::{
        self,
        Debug,
    },
    sync::Arc,
};

use crate::dictionary::Dictionary;

#[derive(Clone, Default)]
pub struct DictGroup {
    dicts: Vec<Arc<dyn Dictionary>>,
    max_key_length: usize,
}

impl DictGroup {
    pub fn new(dicts: Vec<Arc<dyn Dictionary>>) -> Self {
        let max_key_length = dicts.iter().map(|d| d.max_key_length()).max().unwrap_or(0);
        Self {
            dicts,
            max_key_length,
        }
    }
}

impl Debug for DictGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DictGroup")
            .field("dictionaries_count", &self.dicts.len())
            .field("max_key_length", &self.max_key_length)
            .finish()
    }
}

impl Dictionary for DictGroup {
    fn match_prefix<'a>(&self, word: &'a str) -> Option<(&'a str, Vec<String>)> {
        self.dicts
            .iter()
            .filter_map(|dict| dict.match_prefix(word))
            .fold(None, |acc, item| match acc {
                Some(ref best) if best.0.len() >= item.0.len() => acc,
                _ => Some(item),
            })
    }

    fn max_key_length(&self) -> usize {
        self.max_key_length
    }
}
