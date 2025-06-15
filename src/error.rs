use thiserror::Error;

/// `ferrous-opencc` 库的主错误类型。
#[derive(Error, Debug)]
pub enum OpenCCError {
    /// I/O 错误
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// 解析 JSON 配置文件时的错误
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    /// 与 FST 库相关的错误
    #[error("FST error: {0}")]
    Fst(#[from] fst::Error),

    /// Bincode 反序列化过程中的错误
    #[error("Bincode decoding error: {0}")]
    BincodeDecode(#[from] bincode::error::DecodeError),

    /// Bincode 序列化过程中发生的错误
    #[error("Bincode encoding error: {0}")]
    BincodeEncode(#[from] bincode::error::EncodeError),

    /// 无效的配置格式
    #[error("Invalid configuration format: {0}")]
    InvalidConfig(String),

    /// 在嵌入式资源中找不到指定的配置或词典
    #[error("Configuration or resource not found in embedded data: {0}")]
    ConfigNotFound(String),

    /// 找不到所需的文件
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// 不支持的词典类型
    #[error("Unsupported dictionary type: {0}")]
    UnsupportedDictType(String),

    /// 从文本文件编译词典时发生错误
    #[error("Dictionary compile failed")]
    DictCompileError(#[from] anyhow::Error),
}

/// `ferrous-opencc` 操作的 `Result` 类型别名。
pub type Result<T> = std::result::Result<T, OpenCCError>;
