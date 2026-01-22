mod dict_group;
mod fst_dict;

pub mod embedded {
    include!(concat!(env!("OUT_DIR"), "/embedded_map.rs"));
}

use std::{
    fmt::Debug,
    path::{
        Path,
        PathBuf,
    },
    sync::Arc,
};

use crate::{
    config::DictConfig,
    dictionary::{
        dict_group::DictGroup,
        fst_dict::FstDict,
    },
    error::{
        OpenCCError,
        Result,
    },
};

pub trait Dictionary: Send + Sync + Debug {
    fn match_prefix<'a>(&self, word: &'a str) -> Option<(&'a str, Vec<String>)>;
    fn max_key_length(&self) -> usize;
}

#[allow(dead_code)]
pub struct DictType;

impl DictType {
    pub(super) fn from_config(
        config: &DictConfig,
        config_dir: &Path,
    ) -> Result<Arc<dyn Dictionary>> {
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
                    dicts.push(Self::from_config(dict_config, config_dir)?);
                }
                let dict_group = DictGroup::new(dicts);
                Ok(Arc::new(dict_group))
            }
            _ => Err(OpenCCError::UnsupportedDictType(config.dict_type.clone())),
        }
    }

    pub(super) fn from_config_embedded(config: &DictConfig) -> Result<Arc<dyn Dictionary>> {
        match config.dict_type.as_str() {
            "text" | "ocd2" => {
                let file_name = config.file.as_ref().ok_or_else(|| {
                    OpenCCError::InvalidConfig("'file' not found for 'text' dict".to_string())
                })?;

                let dict_bytes = embedded::EMBEDDED_DICTS
                    .get(file_name.as_str())
                    .ok_or_else(|| OpenCCError::ConfigNotFound(file_name.clone()))?;

                let dict = FstDict::from_ocb_bytes(dict_bytes)?;
                Ok(Arc::new(dict))
            }
            "group" => {
                let dict_configs = config.dicts.as_ref().ok_or_else(|| {
                    OpenCCError::InvalidConfig("'dicts' not found for 'group' dict".to_string())
                })?;
                let mut dicts = Vec::new();
                for dict_config in dict_configs {
                    dicts.push(Self::from_config_embedded(dict_config)?);
                }
                let dict_group = DictGroup::new(dicts);
                Ok(Arc::new(dict_group))
            }
            _ => Err(OpenCCError::UnsupportedDictType(config.dict_type.clone())),
        }
    }
}

fn find_dict_path(file_name: &str, config_dir: &Path) -> Result<PathBuf> {
    let path = config_dir.join(file_name);
    if path.exists() {
        return Ok(path);
    }

    Err(OpenCCError::FileNotFound(path.display().to_string()))
}
