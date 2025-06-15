//! # 纯 Rust 实现的 OpenCC
//!
//! 为繁体中文和简体中文之间提供高性能的转换。
//!
//! ## 示例
//!
//! ```no_run
//! use ferrous_opencc::OpenCC;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // 创建 OpenCC 实例
//! let opencc = OpenCC::new("path/to/your/s2t.json")?;
//!
//! // 转换文本
//! let text = "“开放中文转换”是完全由 Rust 实现的。";
//! let converted = opencc.convert(text);
//!
//! assert_eq!(converted, "「開放中文轉換」是完全由 Rust 實現的。");
//! # Ok(())
//! # }
//! ```

pub mod compiler;
pub mod config;
pub mod conversion;
pub mod dictionary;
pub mod error;
pub mod segmentation;

use config::Config;
use conversion::ConversionChain;
use error::Result;
use segmentation::{Segmentation, SegmentationType};
use std::path::Path;

include!(concat!(env!("OUT_DIR"), "/embedded_map.rs"));

/// 核心的 OpenCC 转换器
pub struct OpenCC {
    /// 配置名称
    name: String,
    /// 分词器
    segmentation: Box<dyn Segmentation>,
    /// 用于执行转换的词典链
    conversion_chain: ConversionChain,
}

impl OpenCC {
    /// 从配置文件创建一个新的 `OpenCC` 实例。
    /// 解析 JSON 配置文件，加载所有必需的词典，并构建转换流程。
    ///
    /// # 参数
    ///
    /// * `config_path`: 指向主 JSON 配置文件（例如 `s2t.json`）的路径
    ///
    /// # 返回
    ///
    /// 一个 `Result`，其中包含新的 `OpenCC` 实例，或者在加载失败时返回错误
    pub fn new<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        // 1. 解析配置文件
        let config = Config::from_file(config_path)?;
        let config_dir = config.get_config_directory();

        // 2. 初始化分词器
        let segmentation = SegmentationType::from_config(&config.segmentation, config_dir)?;

        // 3. 初始化转换链
        let conversion_chain = ConversionChain::from_config(&config.conversion_chain, config_dir)?;

        Ok(Self {
            name: config.name,
            segmentation: segmentation.into_segmenter(),
            conversion_chain,
        })
    }

    // 从嵌入的资源创建 OpenCC 实例
    pub fn from_config_name(name: &str) -> Result<Self> {
        use crate::dictionary::embedded;

        let config_str = embedded::EMBEDDED_CONFIGS
            .get(name)
            .ok_or_else(|| error::OpenCCError::ConfigNotFound(name.to_string()))?;

        // 从字符串解析配置
        let config: Config = config_str.parse()?;

        // 初始化分词器和转换链，并告诉它们使用嵌入式词典
        let segmentation = SegmentationType::from_config_embedded(&config.segmentation)?;
        let conversion_chain = ConversionChain::from_config_embedded(&config.conversion_chain)?;

        Ok(Self {
            name: config.name,
            segmentation: segmentation.into_segmenter(),
            conversion_chain,
        })
    }

    /// 根据加载的配置转换字符串
    ///
    /// # 参数
    ///
    /// * `input`: 需要转换的字符串
    ///
    /// # 返回
    ///
    /// 转换后的字符串
    pub fn convert(&self, input: &str) -> String {
        // 1. 使用分词器进行分词
        let segments = self.segmentation.segment(input);
        // 2. 使用转换链进行转换
        self.conversion_chain.convert(&segments)
    }

    /// 返回当前加载的配置名称
    pub fn name(&self) -> &str {
        &self.name
    }
}
