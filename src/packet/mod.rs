use bytes::{Buf, BytesMut};

pub use error::Error;

/// Clientbound packet handling module.
pub mod client;
/// Error types for packet coding.
pub mod error;
/// Serverbound packet handling module.
pub mod server;

/// Tokio-based codec for async packet processing.
#[cfg(feature = "tokio")]
pub mod codec;

/// Trait for types that can be used as packet payloads.
pub trait PacketPayload {
    /// Returns the unique packet ID for this payload.
    fn id(&self) -> u8;

    /// Creates a payload from a packet ID and the associated data bytes.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the data cannot be parsed into a valid payload.
    fn from_id_and_data(id: u8, data: &[u8]) -> Result<Self, Error>
    where
        Self: Sized;

    /// Serializes the payload into a vector of bytes.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the payload cannot be serialized.
    fn to_data_bytes(&self) -> Result<Vec<u8>, Error>;
}

/// A packet containing a payload.
#[derive(Debug)]
pub struct Packet<P: PacketPayload> {
    payload: P,
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
        let mut payload_bytes = BytesMut::from(payload_bytes);
        let id = payload_bytes.try_get_u8().map_err(|_| Error::MissingPacketId)?;
        let payload = P::from_id_and_data(id, &payload_bytes)?;
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
        let mut payload_bytes = vec![self.payload.id()];
        payload_bytes.extend(self.payload.to_data_bytes()?);
        Ok(payload_bytes)
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
