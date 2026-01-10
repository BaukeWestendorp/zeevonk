#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),
}
