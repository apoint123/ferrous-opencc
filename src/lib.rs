//! # Pure Rust implementation of `OpenCC`
//!
//! Provides high-performance conversion between Traditional Chinese and Simplified Chinese.
//!
//! ## Examples
//!
//! ```
//! use ferrous_opencc::OpenCC;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create OpenCC instance
//! let opencc = OpenCC::from_config(ferrous_opencc::config::BuiltinConfig::S2t)?;
//!
//! // Convert text
//! let text = "开放中文转换是完全由 Rust 实现的。";
//! let converted = opencc.convert(text);
//!
//! assert_eq!(converted, "開放中文轉換是完全由 Rust 實現的。");
//! # Ok(())
//! # }
//! ```

pub mod config;
mod conversion;
mod dictionary;
pub mod error;
#[cfg(not(target_arch = "wasm32"))]
pub mod ffi;

use std::path::Path;

use config::Config;
use conversion::ConversionChain;
use error::Result;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use crate::{
    config::BuiltinConfig,
    dictionary::embedded,
};

/// The core `OpenCC` converter
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct OpenCC {
    /// Configuration name
    name: String,
    /// The dictionary chain used for conversion
    conversion_chain: ConversionChain,
}

impl OpenCC {
    /// Creates a new `OpenCC` instance from a configuration file.
    /// Parses the JSON configuration file, loads all required dictionaries, and builds the
    /// conversion chain.
    ///
    /// # Arguments
    ///
    /// * `config_path`: Path to the main JSON configuration file (e.g., `s2t.json`)
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `OpenCC` instance, or an error if loading fails
    pub fn new<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        let config = Config::from_file(config_path)?;
        let config_dir = config.get_config_directory();

        let conversion_chain = ConversionChain::from_config(&config.conversion_chain, config_dir)?;

        Ok(Self {
            name: config.name,
            conversion_chain,
        })
    }

    /// Creates an `OpenCC` instance from a built-in configuration.
    ///
    /// # Example
    /// ```
    /// use ferrous_opencc::{
    ///     OpenCC,
    ///     config::BuiltinConfig,
    ///     error::Result,
    /// };
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

    /// Converts a string according to the loaded configuration.
    ///
    /// # Arguments
    ///
    /// * `input`: The string to convert
    ///
    /// # Returns
    ///
    /// The converted string
    #[must_use]
    pub fn convert(&self, input: &str) -> String {
        self.conversion_chain.convert(input)
    }

    /// Returns the name of the currently loaded configuration
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl OpenCC {
    /// Creates a new `OpenCC` instance.
    ///
    /// @param {`string`} `config_name` - The name of the built-in configuration to use, e.g.,
    /// "s2t.json". @returns {`OpenCC`} - An `OpenCC` instance.
    /// @throws {`JsValue`} - Throws an error object if configuration loading fails.
    ///
    /// @example
    /// ```javascript
    /// import init, { OpenCC } from './pkg/ferrous_opencc.js';
    ///
    /// async function main() {
    ///   await init();
    ///   try {
    ///     const converter = new OpenCC("s2t.json");
    ///     console.log('Load success:', converter.name);
    ///   } catch (err) {
    ///     console.error('Load failed:', err);
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

    /// Converts a string according to the loaded configuration.
    ///
    /// @param {string} input - The string to convert.
    /// @returns {string} - The converted string.
    ///
    /// @example
    /// ```javascript
    /// const traditionalText = converter.convert("开放中文转换");
    /// console.log(traditionalText); // Expected: 開放中文轉換
    /// ```
    #[wasm_bindgen(js_name = convert)]
    #[must_use]
    pub fn convert_wasm(&self, input: &str) -> String {
        self.convert(input)
    }

    /// Gets the name of the currently loaded configuration.
    ///
    /// @returns {string} - The name of the configuration.
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
