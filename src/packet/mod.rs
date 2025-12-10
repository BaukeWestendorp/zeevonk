pub use client::*;
pub use codec::*;
pub use error::*;
pub use server::*;

mod client;
mod codec;
mod error;
mod server;

/// Trait for types that can be used as packet payloads.
pub trait PacketPayload: serde::Serialize + for<'de> serde::Deserialize<'de> {}

/// A packet containing a payload.
#[derive(Debug)]
pub struct Packet<P: PacketPayload> {
    /// This packet's payload.
    pub payload: P,
}

impl<P: PacketPayload> Packet<P> {
    /// Create a new packet.
    pub fn new(payload: P) -> Self {
        Self { payload }
    }

    /// Decodes a packet from bytes (excluding the length prefix).
    pub fn decode_payload_bytes(payload_bytes: &[u8]) -> Result<Self, PacketError> {
        let payload = rmp_serde::from_slice(payload_bytes).map_err(|_| {
            PacketError::InvalidPayload { message: "failed to decode payload".to_string() }
        })?;
        let packet = Packet { payload };
        Ok(packet)
    }

    /// Encodes a packet into bytes (excluding the length prefix).
    pub fn encode_payload_bytes(&self) -> Result<Vec<u8>, PacketError> {
        rmp_serde::to_vec(&self.payload).map_err(|_| PacketError::InvalidPayload {
            message: "failed to encode payload".to_string(),
        })
    }
}
