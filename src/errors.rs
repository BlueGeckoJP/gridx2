use std::path::StripPrefixError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Config error: {0}")]
    Config(String),

    #[error("File system error: {0}")]
    Fs(#[from] std::io::Error),

    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("State lock error: {0}")]
    StateLock(String),

    #[error("TOML parse error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("Path error: {0}")]
    Path(String),

    #[error("Path strip prefix error: {0}")]
    StripPrefix(#[from] StripPrefixError),

    #[error("Directory walk error: {0}")]
    WalkDir(#[from] walkdir::Error),
}

pub type AppResult<T> = Result<T, AppError>;
