#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Missing packet size")]
    MissingPacketSize,
    #[error("Missing packet id")]
    MissingPacketId,
    #[error("Invalid packet id: {0}")]
    InvalidPacketId(u8),
    #[error("Packet too large: {0} bytes")]
    PacketTooLarge(usize),
    #[error(
        "Packet size mismatch: found {expected} bytes in prefix, but actual payload length is {found}"
    )]
    PacketSizeMismatch { expected: u32, found: u32 },
    #[error("Invalid payload: {message}")]
    InvalidPayload { message: String },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
