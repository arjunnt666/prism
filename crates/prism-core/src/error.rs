use thiserror::Error;

pub type Result<T> = std::result::Result<T, PrismError>;

#[derive(Debug, Error)]
pub enum PrismError {
    #[error("not found: {0}")] NotFound(String),
    #[error("parse: {0}")] Parse(String),
    #[error("collect: {0}")] Collect(String),
    #[error("store: {0}")] Store(String),
    #[error("invalid: {0}")] Invalid(String),
    #[error("provider: {0}")] Provider(String),
    #[error("io: {0}")] Io(String),
    #[error("internal: {0}")] Internal(String),
}

impl From<std::io::Error> for PrismError {
    fn from(e: std::io::Error) -> Self { PrismError::Io(e.to_string()) }
}

impl From<serde_json::Error> for PrismError {
    fn from(e: serde_json::Error) -> Self { PrismError::Parse(e.to_string()) }
}
