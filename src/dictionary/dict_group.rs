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
    fn match_prefix<'a, 'b>(&'a self, word: &'b str) -> Option<(&'b str, &'a str)> {
        let max_possible = self.max_key_length.min(word.len());
        let mut best: Option<(&'b str, &'a str)> = None;

        for dict in &self.dicts {
            if let Some(item) = dict.match_prefix(word) {
                let item_len = item.0.len();
                if best.is_none() || item_len > best.unwrap().0.len() {
                    best = Some(item);
                    if item_len == max_possible {
                        break;
                    }
                }
            }
        }

        best
    }

    fn max_key_length(&self) -> usize {
        self.max_key_length
    }
}
