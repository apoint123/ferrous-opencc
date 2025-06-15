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
    /// 通过查询所有子词典来查找最长的前缀匹配。
    /// 该方法会遍历组内的所有词典，对每一个都执行前缀匹配，
    /// 然后从所有结果中返回拥有最长键的那一个。
    fn match_prefix<'a, 'b>(&'a self, word: &'b str) -> Option<(&'b str, &'a [Arc<str>])> {
        // 1. 遍历所有子词典，并对每个词典进行前缀匹配
        //    使用 `filter_map` 收集所有有效的匹配结果
        // 2. 使用 `max_by_key`，根据匹配到的键的长度，从所有结果中找出最长的那一个
        self.dicts
            .iter()
            .filter_map(|dict| dict.match_prefix(word))
            .max_by_key(|(key, _values)| key.len())
    }

    fn max_key_length(&self) -> usize {
        self.max_key_length
    }
}
