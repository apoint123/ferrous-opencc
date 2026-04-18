use thiserror::Error;

#[derive(Error, Debug)]
pub enum OpenCCError {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Error parsing JSON configuration file
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    /// Error related to FST library
    #[error("FST error: {0}")]
    Fst(#[from] fst::Error),

    /// Invalid rkyv dictionary format
    #[error("Invalid rkyv dictionary format: {0}")]
    InvalidDictFormat(#[from] rkyv::rancor::Error),

    /// Invalid configuration format
    #[error("Invalid configuration format: {0}")]
    InvalidConfig(String),

    /// Configuration or dictionary not found in embedded resources
    #[error("Configuration or resource not found in embedded data: {0}")]
    ConfigNotFound(String),

    /// Required file not found
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Unsupported dictionary type
    #[error("Unsupported dictionary type: {0}")]
    UnsupportedDictType(String),

    /// Error compiling dictionary from text file
    #[error("Dictionary compile failed")]
    DictCompileError(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, OpenCCError>;
