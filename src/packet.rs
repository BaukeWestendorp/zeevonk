use std::io;

use bytes::{Buf, BytesMut};

use crate::dmx::Multiverse;

/// Packets sent from the server to the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientboundPacket {
    IntervalDmxOutput,

    ResponseLayout,
    ResponseDmxOutput(Multiverse),
    ResponseTriggers,
    ResponseAttributeValues,
    ResponseSetAttributeValues,
}

impl ClientboundPacket {
    pub fn id(&self) -> u8 {
        match self {
            Self::IntervalDmxOutput => 0,

            Self::ResponseLayout => 1,
            Self::ResponseDmxOutput(_) => 2,
            Self::ResponseTriggers => 3,
            Self::ResponseAttributeValues => 4,
            Self::ResponseSetAttributeValues => 5,
        }
    }

    /// Decodes a clientbound from bytes (excluding the length prefix).
    pub fn decode_payload_bytes(payload_bytes: &[u8]) -> Result<Self, Error> {
        let mut payload_bytes = BytesMut::from(payload_bytes);

        let id = payload_bytes.try_get_u8().map_err(|_| Error::MissingPacketId)?;

        let packet = match id {
            0 => Self::IntervalDmxOutput,

            1 => Self::ResponseLayout,
            2 => {
                let multiverse: Multiverse =
                    rmp_serde::from_slice(&payload_bytes).map_err(|_| Error::InvalidPayload {
                        message: "failed to decode multiverse".to_string(),
                    })?;
                Self::ResponseDmxOutput(multiverse)
            }
            3 => Self::ResponseTriggers,
            4 => Self::ResponseAttributeValues,
            5 => Self::ResponseSetAttributeValues,

            _ => return Err(Error::UnknownPacketId),
        };

        Ok(packet)
    }

    /// Decodes a clientbound packet from bytes including the length prefix (u32 little-endian).
    /// Returns the decoded packet and the number of bytes consumed.
    pub fn decode_packet_bytes(packet_bytes: &[u8]) -> Result<(Self, usize), Error> {
        let mut packet_bytes = BytesMut::from(packet_bytes);

        let payload_length =
            packet_bytes.try_get_u32_le().map_err(|_| Error::MissingPacketId)? as usize;

        if packet_bytes.len() < 4 + payload_length {
            return Err(Error::PacketSizeMismatch {
                expected: payload_length as u32,
                found: packet_bytes.len() as u32,
            });
        }

        let payload_bytes = &packet_bytes[4..4 + payload_length];
        let packet = Self::decode_payload_bytes(payload_bytes)?;
        Ok((packet, 4 + payload_length))
    }

    /// Encodes a clientbound into bytes (excluding the length prefix).
    pub fn encode_payload_bytes(&self) -> Result<Vec<u8>, Error> {
        let mut payload_bytes = vec![self.id()];

        match self {
            Self::IntervalDmxOutput => {}

            Self::ResponseLayout => {}
            Self::ResponseDmxOutput(multiverse) => {
                let multiverse_bytes = rmp_serde::to_vec(multiverse).map_err(|_| {
                    Error::InvalidPayload { message: "failed to encode multiverse".to_string() }
                })?;

                payload_bytes.extend(multiverse_bytes);
            }
            Self::ResponseTriggers => {}
            Self::ResponseAttributeValues => {}
            Self::ResponseSetAttributeValues => {}
        }

        Ok(payload_bytes)
    }

    /// Encodes a clientbound into bytes including the length prefix (u32 LE).
    pub fn encode_packet_bytes(&self) -> Result<Vec<u8>, Error> {
        let payload_bytes = self.encode_payload_bytes()?;
        let length = payload_bytes.len() as u32;
        let mut out = Vec::with_capacity(4 + payload_bytes.len());
        out.extend_from_slice(&length.to_le_bytes());
        out.extend_from_slice(&payload_bytes);
        Ok(out)
    }
}

/// Packets sent from the client to the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerboundPacket {
    RequestLayout,
    RequestDmxOutput,
    RequestTriggers,
    RequestAttributeValues,
    RequestSetAttributeValues,
}

impl ServerboundPacket {
    pub fn id(&self) -> u8 {
        match self {
            Self::RequestLayout => 0,
            Self::RequestDmxOutput => 1,
            Self::RequestTriggers => 2,
            Self::RequestAttributeValues => 3,
            Self::RequestSetAttributeValues => 4,
        }
    }

    /// Decodes a serverbound from bytes (excluding the length prefix).
    pub fn decode_payload_bytes(_payload_bytes: &[u8]) -> Result<Self, Error> {
        let mut payload_bytes = BytesMut::from(_payload_bytes);

        let id = payload_bytes.try_get_u8().map_err(|_| Error::MissingPacketId)?;

        let packet = match id {
            0 => Self::RequestLayout,
            1 => Self::RequestDmxOutput,
            2 => Self::RequestTriggers,
            3 => Self::RequestAttributeValues,
            4 => Self::RequestSetAttributeValues,

            _ => return Err(Error::UnknownPacketId),
        };

        Ok(packet)
    }

    /// Decodes a serverbound packet from bytes including the length prefix (u32 little-endian).
    /// Returns the decoded packet and the number of bytes consumed.
    pub fn decode_packet_bytes(packet_bytes: &[u8]) -> Result<(Self, usize), Error> {
        let mut packet_bytes = BytesMut::from(packet_bytes);

        let payload_length =
            packet_bytes.try_get_u32_le().map_err(|_| Error::MissingPacketId)? as usize;

        if packet_bytes.len() < 4 + payload_length {
            return Err(Error::PacketSizeMismatch {
                expected: payload_length as u32,
                found: packet_bytes.len() as u32,
            });
        }

        let payload_bytes = &packet_bytes[4..4 + payload_length];
        let packet = Self::decode_payload_bytes(payload_bytes)?;
        Ok((packet, 4 + payload_length))
    }

    /// Encodes a serverbound into bytes (excluding the length prefix).
    pub fn encode_payload_bytes(&self) -> Result<Vec<u8>, Error> {
        let payload_bytes = vec![self.id()];

        Ok(payload_bytes)
    }

    /// Encodes a serverbound into bytes including the length prefix (u32 LE).
    pub fn encode_packet_bytes(&self) -> Result<Vec<u8>, Error> {
        let payload_bytes = self.encode_payload_bytes()?;
        let length = payload_bytes.len() as u32;
        let mut out = Vec::with_capacity(4 + payload_bytes.len());
        out.extend_from_slice(&length.to_le_bytes());
        out.extend_from_slice(&payload_bytes);
        Ok(out)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Missing packet id")]
    MissingPacketId,
    #[error("Unknown packet id")]
    UnknownPacketId,
    #[error("Packet too large: {0} bytes")]
    PacketTooLarge(usize),
    #[error(
        "Packet size mismatch: found {expected} bytes in prefix, but actual payload length is {found}"
    )]
    PacketSizeMismatch { expected: u32, found: u32 },
    #[error("Invalid payload: {message}")]
    InvalidPayload { message: String },
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
}

pub(crate) mod codec {
    use bytes::BytesMut;
    use tokio_util::codec::{Decoder, Encoder};

    use crate::packet::{ClientboundPacket, ServerboundPacket};

    pub struct ServerboundPacketDecoder;

    const MAX: usize = 8 * 1024 * 1024;

    impl Decoder for ServerboundPacketDecoder {
        type Item = ServerboundPacket;
        type Error = super::Error;

        fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
            if src.len() < 4 {
                // Not enough payload to read length marker.
                return Ok(None);
            }

            // Read length marker.
            let mut length_bytes = [0u8; 4];
            length_bytes.copy_from_slice(&src[..4]);
            let payload_length = u32::from_le_bytes(length_bytes) as usize;

            // Check that the length is not too large to avoid a denial of
            // service attack where the server runs out of memory.
            if payload_length > MAX {
                return Err(Self::Error::PacketTooLarge(payload_length));
            }

            if src.len() < 4 + payload_length {
                // The full packet has not yet arrived.
                //
                // We reserve more space in the buffer. This is not strictly
                // necessary, but is a good idea performance-wise.
                src.reserve(4 + payload_length - src.len());

                return Ok(None);
            }

            Ok(Some(ServerboundPacket::decode_payload_bytes(
                src.split_to(4 + payload_length).as_ref(),
            )?))
        }
    }

    pub struct ClientboundPacketEncoder;

    impl Encoder<ClientboundPacket> for ClientboundPacketEncoder {
        type Error = super::Error;

        fn encode(
            &mut self,
            packet: ClientboundPacket,
            dst: &mut BytesMut,
        ) -> Result<(), Self::Error> {
            let payload_bytes = packet.encode_payload_bytes()?;

            // Check if the length of the length prefix + payload bytes is within the limit.
            if 4 + payload_bytes.len() > MAX {
                return Err(super::Error::PacketTooLarge(payload_bytes.len()));
            }

            // Convert the length into a byte array.
            // The cast to u32 cannot overflow due to the length check above.
            let len_slice = u32::to_le_bytes(payload_bytes.len() as u32);

            // Reserve space in the buffer.
            dst.reserve(4 + payload_bytes.len());

            // Write the length prefix and packet payload to the buffer.
            dst.extend_from_slice(&len_slice);
            dst.extend_from_slice(&payload_bytes);

            Ok(())
        }
    }
}
