//! 定义了 `DictGroup`，一个作为其他词典集合的特殊词典

use crate::dictionary::Dictionary;
use std::fmt::{self, Debug};
use std::sync::Arc;

/// 一个集合词典
#[derive(Clone, Default)]
pub struct DictGroup {
    /// 该词典组包含的子词典向量
    dicts: Vec<Arc<dyn Dictionary>>,
    /// 所有子词典中最长的键的长度，用于优化
    max_key_length: usize,
}

impl DictGroup {
    /// 从一个包含多个词典的向量中创建一个新的 `DictGroup`
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
        // 由于无法直接打印 `dyn Dictionary`, 只打印一些有用的元信息
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
