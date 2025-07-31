use crate::error::{OpenCCError, Result};
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

/// 顶层的 JSON 配置结构
#[derive(Deserialize, Debug)]
pub struct Config {
    /// 转换配置的名称
    pub name: String,
    /// 转换步骤链
    pub conversion_chain: Vec<ConversionNodeConfig>,

    /// 配置文件所在的目录
    #[serde(skip)]
    config_directory: PathBuf,
}

/// 所有内置的 OpenCC 配置
#[repr(i32)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinConfig {
    /// 简体到繁体
    S2t = 0,
    /// 繁体到简体
    T2s = 1,
    /// 简体到台湾正体
    S2tw = 2,
    /// 台湾正体到简体
    Tw2s = 3,
    /// 简体到香港繁体
    S2hk = 4,
    /// 香港繁体到简体
    Hk2s = 5,
    /// 简体到台湾正体（包含词汇转换）
    S2twp = 6,
    /// 台湾正体（包含词汇转换）到简体
    Tw2sp = 7,
    /// 繁体到台湾正体
    T2tw = 8,
    /// 台湾正体到繁体
    Tw2t = 9,
    /// 繁体到香港繁体
    T2hk = 10,
    /// 香港繁体到繁体
    Hk2t = 11,
    /// 日语新字体到繁体
    Jp2t = 12,
    /// 繁体到日语新字体
    T2jp = 13,
}

impl BuiltinConfig {
    /// 将枚举成员转换为对应的文件名字符串
    pub fn to_filename(&self) -> &'static str {
        match self {
            BuiltinConfig::S2t => "s2t.json",
            BuiltinConfig::T2s => "t2s.json",
            BuiltinConfig::S2tw => "s2tw.json",
            BuiltinConfig::Tw2s => "tw2s.json",
            BuiltinConfig::S2hk => "s2hk.json",
            BuiltinConfig::Hk2s => "hk2s.json",
            BuiltinConfig::S2twp => "s2twp.json",
            BuiltinConfig::Tw2sp => "tw2sp.json",
            BuiltinConfig::T2tw => "t2tw.json",
            BuiltinConfig::Tw2t => "tw2t.json",
            BuiltinConfig::T2hk => "t2hk.json",
            BuiltinConfig::Hk2t => "hk2t.json",
            BuiltinConfig::Jp2t => "jp2t.json",
            BuiltinConfig::T2jp => "t2jp.json",
        }
    }

    /// 从文件名字符串转换为对应的枚举成员
    pub fn from_filename(filename: &str) -> Result<Self> {
        match filename {
            "s2t.json" => Ok(BuiltinConfig::S2t),
            "t2s.json" => Ok(BuiltinConfig::T2s),
            "s2tw.json" => Ok(BuiltinConfig::S2tw),
            "tw2s.json" => Ok(BuiltinConfig::Tw2s),
            "s2hk.json" => Ok(BuiltinConfig::S2hk),
            "hk2s.json" => Ok(BuiltinConfig::Hk2s),
            "s2twp.json" => Ok(BuiltinConfig::S2twp),
            "tw2sp.json" => Ok(BuiltinConfig::Tw2sp),
            "t2tw.json" => Ok(BuiltinConfig::T2tw),
            "tw2t.json" => Ok(BuiltinConfig::Tw2t),
            "t2hk.json" => Ok(BuiltinConfig::T2hk),
            "hk2t.json" => Ok(BuiltinConfig::Hk2t),
            "jp2t.json" => Ok(BuiltinConfig::Jp2t),
            "t2jp.json" => Ok(BuiltinConfig::T2jp),
            _ => Err(OpenCCError::ConfigNotFound(filename.to_string())),
        }
    }
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

impl FromStr for Config {
    type Err = OpenCCError;

    fn from_str(s: &str) -> Result<Self> {
        let config: Config = serde_json::from_str(s)?;
        Ok(config)
    }
}
