use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to serialize showfile: {message}")]
    SerializationError { message: String },
    #[error("failed to deserialize showfile: {message}")]
    DeserializationError { message: String },
    #[error("missing or invalid directory: {0}")]
    InvalidDirectory(String),
}
