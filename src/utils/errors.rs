use thiserror::Error;

#[derive(Error, Debug)]
pub enum CustomError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    // #[error("Custom error message: {0}")]
    // Custom(String),
    #[error("DawnAPIError message: {0}")]
    DawnAPIError(String),

    #[error("EmailAPIError message: {0}")]
    EmailAPIError(String),

    #[error("EmailFileError message: {0}")]
    EmailFileError(String),

    #[error("SettingFileError message: {0}")]
    SettingFileError(String),

    #[error("HtmlParseError message: {0}")]
    HtmlParseError(String),

    #[error("CustomRedisError message: {0}")]
    CustomRedisError(String),

    #[error("RedisInfoError message: {0}")]
    RedisInfoError(String),

    #[error("CaptchaError message: {0}")]
    CaptchaError(String),
}
