use bytes::{Buf, BufMut as _, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::packet::{Packet, PacketPayload};

pub const MAX_PACKET_LENGTH: usize = 8 * 1024 * 1024;

#[derive(Default)]
pub struct PacketEncoder;

impl<P: PacketPayload> Encoder<Packet<P>> for PacketEncoder {
    type Error = super::Error;

    fn encode(&mut self, packet: Packet<P>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let payload_bytes = packet.encode_payload_bytes()?;

        // Check if the length of the length prefix + payload bytes is within the limit.
        if 4 + payload_bytes.len() > MAX_PACKET_LENGTH {
            return Err(super::Error::PacketTooLarge(payload_bytes.len()));
        }

        // Reserve space in the buffer.
        dst.reserve(4 + payload_bytes.len());

        // Write the length prefix using BufMut and packet payload to the buffer.
        dst.put_u32_le(payload_bytes.len() as u32);
        dst.extend_from_slice(&payload_bytes);

        Ok(())
    }
}

pub struct PacketDecoder<P: PacketPayload> {
    marker: std::marker::PhantomData<P>,
}

impl<P: PacketPayload> Default for PacketDecoder<P> {
    fn default() -> Self {
        Self { marker: std::marker::PhantomData::default() }
    }
}

impl<P: PacketPayload> Decoder for PacketDecoder<P> {
    type Item = Packet<P>;
    type Error = super::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            // Not enough payload to read length prefix.
            return Ok(None);
        }

        // Read length prefix.
        let payload_length =
            src.try_get_u32_le().map_err(|_| Self::Error::MissingPacketSize)? as usize;

        // Check that the length is not too large to avoid a denial of
        // service attack where the server runs out of memory.
        if payload_length > MAX_PACKET_LENGTH {
            return Err(Self::Error::PacketTooLarge(payload_length));
        }

        if src.len() < payload_length {
            // The full packet has not yet arrived.
            //
            // We reserve more space in the buffer. This is not strictly
            // necessary, but is a good idea performance-wise.
            src.reserve(payload_length - src.len());

            return Ok(None);
        }

        Ok(Some(Packet::decode_payload_bytes(&src)?))
    }
}
