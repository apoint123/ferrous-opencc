use std::{
    fs::File,
    io::BufReader,
    path::{
        Path,
        PathBuf,
    },
    str::FromStr,
};

use serde::{
    Deserialize,
    Serialize,
};

use crate::error::{
    OpenCCError,
    Result,
};

/// Top-level JSON configuration structure
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    /// Name of the conversion configuration
    pub name: String,
    /// Chain of conversion steps
    pub conversion_chain: Vec<ConversionNodeConfig>,

    /// Directory where the configuration file is located
    #[serde(skip)]
    directory: PathBuf,
}

/// All built-in `OpenCC` configurations
#[repr(i32)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinConfig {
    /// Simplified to Traditional
    #[cfg(feature = "s2t-conversion")]
    S2t = 0,
    /// Traditional to Simplified
    #[cfg(feature = "t2s-conversion")]
    T2s = 1,
    /// Simplified to Traditional (Taiwan)
    #[cfg(feature = "s2t-conversion")]
    S2tw = 2,
    /// Traditional (Taiwan) to Simplified
    #[cfg(feature = "t2s-conversion")]
    Tw2s = 3,
    /// Simplified to Traditional (Hong Kong)
    #[cfg(feature = "s2t-conversion")]
    S2hk = 4,
    /// Traditional (Hong Kong) to Simplified
    #[cfg(feature = "t2s-conversion")]
    Hk2s = 5,
    /// Simplified to Traditional (Taiwan) (including vocabulary conversion)
    #[cfg(feature = "s2t-conversion")]
    S2twp = 6,
    /// Traditional (Taiwan) (including vocabulary conversion) to Simplified
    #[cfg(feature = "t2s-conversion")]
    Tw2sp = 7,
    /// Traditional to Traditional (Taiwan)
    #[cfg(feature = "t2s-conversion")]
    T2tw = 8,
    /// Traditional (Taiwan) to Traditional
    #[cfg(feature = "s2t-conversion")]
    Tw2t = 9,
    /// Traditional to Traditional (Hong Kong)
    #[cfg(feature = "s2t-conversion")]
    T2hk = 10,
    /// Traditional (Hong Kong) to Traditional
    #[cfg(feature = "t2s-conversion")]
    Hk2t = 11,
    /// Japanese Shinjitai to Traditional
    #[cfg(feature = "japanese-conversion")]
    Jp2t = 12,
    /// Traditional to Japanese Shinjitai
    #[cfg(feature = "japanese-conversion")]
    T2jp = 13,
}

impl BuiltinConfig {
    /// Converts the enum variant to the corresponding filename string
    #[must_use]
    pub const fn to_filename(&self) -> &'static str {
        match self {
            #[cfg(feature = "s2t-conversion")]
            Self::S2t => "s2t.json",
            #[cfg(feature = "t2s-conversion")]
            Self::T2s => "t2s.json",
            #[cfg(feature = "s2t-conversion")]
            Self::S2tw => "s2tw.json",
            #[cfg(feature = "t2s-conversion")]
            Self::Tw2s => "tw2s.json",
            #[cfg(feature = "s2t-conversion")]
            Self::S2hk => "s2hk.json",
            #[cfg(feature = "t2s-conversion")]
            Self::Hk2s => "hk2s.json",
            #[cfg(feature = "s2t-conversion")]
            Self::S2twp => "s2twp.json",
            #[cfg(feature = "t2s-conversion")]
            Self::Tw2sp => "tw2sp.json",
            #[cfg(feature = "t2s-conversion")]
            Self::T2tw => "t2tw.json",
            #[cfg(feature = "s2t-conversion")]
            Self::Tw2t => "tw2t.json",
            #[cfg(feature = "s2t-conversion")]
            Self::T2hk => "t2hk.json",
            #[cfg(feature = "t2s-conversion")]
            Self::Hk2t => "hk2t.json",
            #[cfg(feature = "japanese-conversion")]
            Self::Jp2t => "jp2t.json",
            #[cfg(feature = "japanese-conversion")]
            Self::T2jp => "t2jp.json",
        }
    }

    /// Converts a filename string to the corresponding enum variant
    pub fn from_filename(filename: &str) -> Result<Self> {
        match filename {
            #[cfg(feature = "s2t-conversion")]
            "s2t.json" => Ok(Self::S2t),
            #[cfg(feature = "t2s-conversion")]
            "t2s.json" => Ok(Self::T2s),
            #[cfg(feature = "s2t-conversion")]
            "s2tw.json" => Ok(Self::S2tw),
            #[cfg(feature = "t2s-conversion")]
            "tw2s.json" => Ok(Self::Tw2s),
            #[cfg(feature = "s2t-conversion")]
            "s2hk.json" => Ok(Self::S2hk),
            #[cfg(feature = "t2s-conversion")]
            "hk2s.json" => Ok(Self::Hk2s),
            #[cfg(feature = "s2t-conversion")]
            "s2twp.json" => Ok(Self::S2twp),
            #[cfg(feature = "t2s-conversion")]
            "tw2sp.json" => Ok(Self::Tw2sp),
            #[cfg(feature = "t2s-conversion")]
            "t2tw.json" => Ok(Self::T2tw),
            #[cfg(feature = "s2t-conversion")]
            "tw2t.json" => Ok(Self::Tw2t),
            #[cfg(feature = "s2t-conversion")]
            "t2hk.json" => Ok(Self::T2hk),
            #[cfg(feature = "t2s-conversion")]
            "hk2t.json" => Ok(Self::Hk2t),
            #[cfg(feature = "japanese-conversion")]
            "jp2t.json" => Ok(Self::Jp2t),
            #[cfg(feature = "japanese-conversion")]
            "t2jp.json" => Ok(Self::T2jp),
            _ => Err(OpenCCError::ConfigNotFound(filename.to_string())),
        }
    }
}

/// A node in the conversion chain
///
/// Each node corresponds to a dictionary-based conversion step
#[derive(Serialize, Deserialize, Debug)]
pub struct ConversionNodeConfig {
    /// The dictionary to use for this conversion step
    pub dict: DictConfig,
}

/// Represents a dictionary configuration, which can be a single dictionary file or a group of
/// dictionaries
#[derive(Serialize, Deserialize, Debug)]
pub struct DictConfig {
    /// Dictionary type, e.g., "text" or "group"
    #[serde(rename = "type")]
    pub dict_type: String,
    /// Dictionary filename (for `type: "text"`)
    pub file: Option<String>,
    /// List of sub-dictionaries (for `type: "group"`)
    pub dicts: Option<Vec<Self>>,
}

impl Config {
    /// Loads and parses configuration from a JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let file = File::open(path)
            .map_err(|e| OpenCCError::FileNotFound(format!("{}: {}", path.display(), e)))?;
        let reader = BufReader::new(file);
        let mut config: Self = serde_json::from_reader(reader)?;

        config.directory = path.parent().unwrap_or_else(|| Path::new("")).to_path_buf();

        Ok(config)
    }

    /// Gets the directory where the configuration file is located
    #[must_use]
    pub fn get_config_directory(&self) -> &Path {
        &self.directory
    }
}

impl FromStr for Config {
    type Err = OpenCCError;

    fn from_str(s: &str) -> Result<Self> {
        let config: Self = serde_json::from_str(s)?;
        Ok(config)
    }
}
