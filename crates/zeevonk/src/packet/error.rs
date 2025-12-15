#[derive(Debug, thiserror::Error)]
/// Errors that can occur during packet processing.
pub enum Error {
    #[error("packet too large: {0} bytes")]
    PacketTooLarge(usize),

    /// The payload is invalid.
    #[error("invalid payload {message}")]
    InvalidPayload { message: String },

    /// An I/O error occurred.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
}
