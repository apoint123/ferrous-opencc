use crate::error::{OpenCCError, Result};
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

/// 顶层的 JSON 配置结构
#[derive(Deserialize, Debug)]
pub struct Config {
    /// 转换配置的名称
    pub name: String,
    /// 分词相关的配置
    pub segmentation: SegmentationConfig,
    /// 转换步骤链
    pub conversion_chain: Vec<ConversionNodeConfig>,

    /// 配置文件所在的目录
    #[serde(skip)]
    config_directory: PathBuf,
}

/// 配置中的分词部分
#[derive(Deserialize, Debug)]
pub struct SegmentationConfig {
    /// 使用的分词器类型，例如 "mm" (最大匹配)
    #[serde(rename = "type")]
    pub seg_type: String,
    /// 分词所使用的词典
    pub dict: DictConfig,
}

/// 转换链中的一个节点
/// 每个节点对应一个基于词典的转换步骤
#[derive(Deserialize, Debug)]
pub struct ConversionNodeConfig {
    /// 此转换步骤要使用的词典
    pub dict: DictConfig,
}

/// 代表一个词典配置，可以是一个单独的词典文件，也可以是一组词典
#[derive(Deserialize, Debug)]
pub struct DictConfig {
    /// 词典的类型，例如 "text" 或 "group"
    #[serde(rename = "type")]
    pub dict_type: String,
    /// 词典文件名 (用于 `type: "text"`)
    pub file: Option<String>,
    /// 子词典列表 (用于 `type: "group"`)。
    pub dicts: Option<Vec<DictConfig>>,
}

impl Config {
    /// 从 JSON 文件加载并解析配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let file = File::open(path)
            .map_err(|e| OpenCCError::FileNotFound(format!("{}: {}", path.display(), e)))?;
        let reader = BufReader::new(file);
        let mut config: Config = serde_json::from_reader(reader)?;

        // 保存配置文件的父目录
        config.config_directory = path.parent().unwrap_or_else(|| Path::new("")).to_path_buf();

        Ok(config)
    }

    /// 获取配置文件所在的目录
    pub fn get_config_directory(&self) -> &Path {
        &self.config_directory
    }
}
