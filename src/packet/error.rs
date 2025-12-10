#[derive(Debug, thiserror::Error)]
/// Errors that can occur during packet processing.
pub enum PacketError {
    /// The packet is too large.
    #[error("Packet too large: {0} bytes")]
    PacketTooLarge(usize),

    /// The payload is invalid.
    #[error("Invalid payload: {message}")]
    InvalidPayload {
        /// A message describing why the payload is invalid.
        message: String,
    },

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
