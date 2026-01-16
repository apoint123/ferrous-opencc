//! # 纯 Rust 实现的 `OpenCC`
//!
//! 为繁体中文和简体中文之间提供高性能的转换。
//!
//! ## 示例
//!
//! ```
//! use ferrous_opencc::OpenCC;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // 创建 OpenCC 实例
//! let opencc = OpenCC::from_config(ferrous_opencc::config::BuiltinConfig::S2t)?;
//!
//! // 转换文本
//! let text = "开放中文转换是完全由 Rust 实现的。";
//! let converted = opencc.convert(text);
//!
//! assert_eq!(converted, "開放中文轉換是完全由 Rust 實現的。");
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod conversion;
pub mod dictionary;
pub mod error;
#[cfg(not(target_arch = "wasm32"))]
pub mod ffi;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use config::Config;
use conversion::ConversionChain;
use error::Result;
use std::path::Path;

use crate::{config::BuiltinConfig, dictionary::embedded};

/// 核心的 `OpenCC` 转换器
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct OpenCC {
    /// 配置名称
    name: String,
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

        // 2. 初始化转换链
        let conversion_chain = ConversionChain::from_config(&config.conversion_chain, config_dir)?;

        Ok(Self {
            name: config.name,
            conversion_chain,
        })
    }

    /// 从内置的配置创建 `OpenCC` 实例。
    ///
    /// # 示例
    /// ```
    /// use ferrous_opencc::{OpenCC, config::BuiltinConfig, error::Result};
    ///
    /// fn main() -> Result<()> {
    ///     let opencc = OpenCC::from_config(BuiltinConfig::S2t)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn from_config(config_enum: BuiltinConfig) -> Result<Self> {
        let name = config_enum.to_filename();
        let config_str = embedded::EMBEDDED_CONFIGS
            .get(name)
            .ok_or_else(|| error::OpenCCError::ConfigNotFound(name.to_string()))?;

        let config: Config = config_str.parse()?;

        let conversion_chain = ConversionChain::from_config_embedded(&config.conversion_chain)?;

        Ok(Self {
            name: config.name,
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
    #[must_use]
    pub fn convert(&self, input: &str) -> String {
        self.conversion_chain.convert(input)
    }

    /// 返回当前加载的配置名称
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl OpenCC {
    /// 创建一个新的 `OpenCC` 实例。
    ///
    /// @param {`string`} `config_name` - 要使用的内置配置名称, 例如 "s2t.json"。
    /// @returns {`OpenCC`} - 一个 `OpenCC` 实例。
    /// @throws {`JsValue`} - 如果配置加载失败，则抛出一个错误对象。
    ///
    /// @example
    /// ```javascript
    /// import init, { OpenCC } from './pkg/ferrous_opencc.js';
    ///
    /// async function main() {
    ///   await init();
    ///   try {
    ///     const converter = new OpenCC("s2t.json");
    ///     console.log('加载成功:', converter.name);
    ///   } catch (err) {
    ///     console.error('加载失败:', err);
    ///   }
    /// }
    /// main();
    /// ```
    #[wasm_bindgen(constructor)]
    pub fn new_wasm(config_name: &str) -> std::result::Result<Self, JsValue> {
        let config_enum = BuiltinConfig::from_filename(config_name)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Self::from_config(config_enum).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// 根据加载的配置转换字符串。
    ///
    /// @param {string} input - 需要转换的字符串。
    /// @returns {string} - 转换后的字符串。
    ///
    /// @example
    /// ```javascript
    /// const traditionalText = converter.convert("开放中文转换");
    /// console.log(traditionalText); // 预期: 開放中文轉換
    /// ```
    #[wasm_bindgen(js_name = convert)]
    #[must_use]
    pub fn convert_wasm(&self, input: &str) -> String {
        self.convert(input)
    }

    /// 获取当前加载的配置的名称。
    ///
    /// @returns {string} - 配置的名称。
    ///
    /// @example
    /// ```javascript
    /// const configName = converter.name;
    /// console.log(configName);
    /// ```
    #[wasm_bindgen(getter, js_name = name)]
    #[must_use]
    pub fn name_wasm(&self) -> String {
        self.name.clone()
    }
}
