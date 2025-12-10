#[derive(Debug, thiserror::Error)]
/// Errors that can occur during packet processing.
pub enum PacketError {
    /// Packet size prefix is missing from the input.
    #[error("Missing packet size")]
    MissingPacketSize,

    /// Packet ID is missing from the input.
    #[error("Missing packet id")]
    MissingPacketId,

    /// The packet ID is invalid.
    #[error("Invalid packet id: {0}")]
    InvalidPacketId(u8),

    /// The packet is too large.
    #[error("Packet too large: {0} bytes")]
    PacketTooLarge(usize),

    /// The packet size prefix does not match the actual payload length.
    #[error(
        "Packet size mismatch: found {expected} bytes in prefix, but actual payload length is {found}"
    )]
    PacketSizeMismatch {
        /// The expected size from the packet prefix.
        expected: u32,
        /// The actual payload length found.
        found: u32,
    },

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
