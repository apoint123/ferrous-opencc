//! 负责词典处理的模块

pub mod dict_group;
pub mod fst_dict;

pub mod embedded {
    include!(concat!(env!("OUT_DIR"), "/embedded_map.rs"));
}

use crate::config::DictConfig;
use crate::error::{OpenCCError, Result};
use dict_group::DictGroup;
use fst_dict::FstDict;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// 代表词典基本功能的 trait
pub trait Dictionary: Send + Sync + Debug {
    /// 在词典中查找给定单词的最长前缀匹配
    ///
    /// # 返回
    ///
    /// 如果找到匹配，返回一个包含 `(匹配到的键, 匹配到的值列表)` 的元组
    fn match_prefix<'a, 'b>(&'a self, word: &'b str) -> Option<(&'b str, &'a [Arc<str>])>;

    /// 返回词典中的最长键长度，可用于分词算法的优化
    fn max_key_length(&self) -> usize;
}

/// 一个内部枚举，用作词典工厂函数的命名空间
/// 它用于根据配置分发不同词典的加载逻辑。
pub enum DictType {
    Fst(FstDict),
    Group(DictGroup),
}

impl DictType {
    /// 从文件加载字典
    pub fn from_config(config: &DictConfig, config_dir: &Path) -> Result<Arc<dyn Dictionary>> {
        match config.dict_type.as_str() {
            "text" | "ocd2" => {
                let file_name = config.file.as_ref().ok_or_else(|| {
                    OpenCCError::InvalidConfig("'file' not found for 'text' dict".to_string())
                })?;
                let dict_path = find_dict_path(file_name, config_dir)?;
                let dict = FstDict::new(&dict_path)?;
                Ok(Arc::new(dict))
            }
            "group" => {
                let dict_configs = config.dicts.as_ref().ok_or_else(|| {
                    OpenCCError::InvalidConfig("'dicts' not found for 'group' dict".to_string())
                })?;
                let mut dicts = Vec::new();
                for dict_config in dict_configs {
                    // 递归调用 from_config 来构建子词典
                    dicts.push(Self::from_config(dict_config, config_dir)?);
                }
                let dict_group = DictGroup::new(dicts);
                Ok(Arc::new(dict_group))
            }
            _ => Err(OpenCCError::UnsupportedDictType(config.dict_type.clone())),
        }
    }

    /// 从嵌入的资源加载字典
    pub fn from_config_embedded(config: &DictConfig) -> Result<Arc<dyn Dictionary>> {
        match config.dict_type.as_str() {
            "text" | "ocd2" => {
                let file_name = config.file.as_ref().ok_or_else(|| {
                    OpenCCError::InvalidConfig("'file' not found for 'text' dict".to_string())
                })?;

                // 只在嵌入式 map 中查找
                let dict_bytes = embedded::EMBEDDED_DICTS
                    .get(file_name.as_str())
                    .ok_or_else(|| OpenCCError::ConfigNotFound(file_name.to_string()))?;

                let dict = FstDict::from_ocb_bytes(dict_bytes)?;
                Ok(Arc::new(dict))
            }
            "group" => {
                let dict_configs = config.dicts.as_ref().ok_or_else(|| {
                    OpenCCError::InvalidConfig("'dicts' not found for 'group' dict".to_string())
                })?;
                let mut dicts = Vec::new();
                for dict_config in dict_configs {
                    // 递归调用嵌入式方法
                    dicts.push(Self::from_config_embedded(dict_config)?);
                }
                let dict_group = DictGroup::new(dicts);
                Ok(Arc::new(dict_group))
            }
            _ => Err(OpenCCError::UnsupportedDictType(config.dict_type.clone())),
        }
    }
}

/// 一个用于在配置目录中定位词典文件的辅助函数
fn find_dict_path(file_name: &str, config_dir: &Path) -> Result<PathBuf> {
    let path = config_dir.join(file_name);
    if path.exists() {
        return Ok(path);
    }

    Err(OpenCCError::FileNotFound(path.display().to_string()))
}
