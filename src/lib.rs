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

use std::sync::Arc;

use conversion::Converter;
#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use crate::config::BuiltinConfig;
pub use crate::{
    dictionary::FstDict,
    error::Result,
};

macro_rules! load_dict_bytes {
    (
        $target:expr;
        $( #[cfg(feature = $feat:literal)] $variant:ident => $file_name:literal ),* $(,)?
    ) => {
        match $target {
            $(
                #[cfg(feature = $feat)]
                BuiltinConfig::$variant => include_bytes!(concat!(env!("OUT_DIR"), "/", $file_name, ".ocb")),
            )*
        }
    };
}

/// The core `OpenCC` converter
#[cfg_attr(feature = "wasm", wasm_bindgen)]
pub struct OpenCC {
    /// Configuration name
    name: String,
    /// The converter used for conversion
    converter: Converter,
}

impl OpenCC {
    /// Creates a new `OpenCC` instance from a configuration file.
    /// Parses the JSON configuration file, loads all required dictionaries, and builds the
    /// converter.
    ///
    /// # Arguments
    ///
    /// * `config_path`: Path to the main JSON configuration file (e.g., `s2t.json`)
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `OpenCC` instance, or an error if loading fails
    #[cfg(feature = "runtime-compilation")]
    pub fn new<P: AsRef<std::path::Path>>(config_path: P) -> Result<Self> {
        let config_path = config_path.as_ref();
        let config = crate::config::Config::from_file(config_path)?;
        let config_dir = config.get_config_directory();

        let compiler_chain = build_compiler_chain(&config, config_dir);

        let ocb_bytes = ferrous_opencc_compiler::compile_chain(&compiler_chain)
            .map_err(|e| error::OpenCCError::InvalidConfig(e.to_string()))?;

        let dict = FstDict::from_ocb_bytes(&ocb_bytes)?;

        Ok(Self {
            name: config.name,
            converter: Converter::new(Arc::new(dict)),
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
        let json_name = config_enum.to_filename();

        let dict_bytes: &[u8] = load_dict_bytes!(config_enum;
            #[cfg(feature = "s2t-conversion")] S2t => "s2t",
            #[cfg(feature = "t2s-conversion")] T2s => "t2s",

            #[cfg(feature = "s2t-conversion")] S2tw => "s2tw",
            #[cfg(feature = "t2s-conversion")] Tw2s => "tw2s",

            #[cfg(feature = "s2t-conversion")] S2hk => "s2hk",
            #[cfg(feature = "t2s-conversion")] Hk2s => "hk2s",

            #[cfg(feature = "s2t-conversion")] S2twp => "s2twp",
            #[cfg(feature = "t2s-conversion")] Tw2sp => "tw2sp",

            #[cfg(feature = "t2s-conversion")] T2tw => "t2tw",
            #[cfg(feature = "s2t-conversion")] Tw2t => "tw2t",

            #[cfg(feature = "s2t-conversion")] T2hk => "t2hk",
            #[cfg(feature = "t2s-conversion")] Hk2t => "hk2t",

            #[cfg(feature = "japanese-conversion")] Jp2t => "jp2t",
            #[cfg(feature = "japanese-conversion")] T2jp => "t2jp",
        );

        let dict = FstDict::from_ocb_bytes(dict_bytes)?;

        Ok(Self {
            name: json_name.to_string(),
            converter: Converter::new(Arc::new(dict)),
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
        self.converter.convert(input)
    }

    /// Returns the name of the currently loaded configuration
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(feature = "runtime-compilation")]
fn build_compiler_chain(
    config: &crate::config::Config,
    config_dir: &std::path::Path,
) -> ferrous_opencc_compiler::CompilerChain {
    use ferrous_opencc_compiler::{
        CompilerChain,
        CompilerDictGroup,
    };

    let mut groups = Vec::new();
    for node in &config.conversion_chain {
        let mut paths = Vec::new();

        collect_dict_paths(&node.dict, config_dir, &mut paths);

        groups.push(CompilerDictGroup { dict_paths: paths });
    }
    CompilerChain { groups }
}

#[cfg(feature = "runtime-compilation")]
fn collect_dict_paths(
    dict_cfg: &crate::config::DictConfig,
    config_dir: &std::path::Path,
    paths: &mut Vec<std::path::PathBuf>,
) {
    if let Some(file) = &dict_cfg.file {
        let actual_file = file.replace(".ocd2", ".txt");
        paths.push(config_dir.join(actual_file));
    }
    if let Some(dicts) = &dict_cfg.dicts {
        for d in dicts {
            collect_dict_paths(d, config_dir, paths);
        }
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
