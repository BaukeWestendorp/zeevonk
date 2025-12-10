use bytes::{Buf, BytesMut};

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
    pub(crate) payload: P,
}

impl<P: PacketPayload> Packet<P> {
    /// Create a new packet.
    pub fn new(payload: P) -> Self {
        Self { payload }
    }

    /// This packet's payload.
    pub fn payload(&self) -> &P {
        &self.payload
    }

    /// Decodes a packet from bytes (excluding the length prefix).
    pub fn decode_payload_bytes(payload_bytes: &[u8]) -> Result<Self, Error> {
        let payload = rmp_serde::from_slice(payload_bytes).map_err(|_| Error::InvalidPayload {
            message: "failed to decode payload".to_string(),
        })?;
        let packet = Packet { payload };
        Ok(packet)
    }

    /// Decodes a packet from bytes including the length prefix (u32 little-endian).
    /// Returns the decoded packet and the number of bytes consumed.
    pub fn decode_packet_bytes(packet_bytes: &[u8]) -> Result<(Self, usize), Error> {
        let mut packet_bytes = BytesMut::from(packet_bytes);

        let payload_length =
            packet_bytes.try_get_u32_le().map_err(|_| Error::MissingPacketId)? as usize;

        if packet_bytes.len() != payload_length {
            return Err(Error::PacketSizeMismatch {
                expected: payload_length as u32,
                found: packet_bytes.len() as u32,
            });
        }

        let packet = Self::decode_payload_bytes(&packet_bytes)?;
        Ok((packet, 4 + payload_length))
    }

    /// Encodes a packet into bytes (excluding the length prefix).
    pub fn encode_payload_bytes(&self) -> Result<Vec<u8>, Error> {
        rmp_serde::to_vec(self.payload())
            .map_err(|_| Error::InvalidPayload { message: "failed to encode payload".to_string() })
    }

    /// Encodes a packet into bytes including the length prefix (u32 little-endian).
    pub fn encode_packet_bytes(&self) -> Result<Vec<u8>, Error> {
        let payload_bytes = self.encode_payload_bytes()?;
        let length = payload_bytes.len() as u32;
        let mut out = Vec::with_capacity(4 + payload_bytes.len());
        out.extend_from_slice(&length.to_le_bytes());
        out.extend_from_slice(&payload_bytes);
        Ok(out)
    }
}
