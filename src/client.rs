use futures::{SinkExt, StreamExt as _};
use tokio::io;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::dmx::Multiverse;
use crate::gdcs::{Attribute, ClampedValue, FixturePath};
use crate::packet::{
    ClientboundPacketPayload, Packet, PacketDecoder, PacketEncoder, ServerboundPacketPayload,
};
use crate::server::BakedPatch;

/// The Zeevonk client.
pub struct ZeevonkClient {
    packet_reader: FramedRead<OwnedReadHalf, PacketDecoder<ClientboundPacketPayload>>,
    packet_writer: FramedWrite<OwnedWriteHalf, PacketEncoder<ServerboundPacketPayload>>,
}
impl ZeevonkClient {
    /// Connects to a Zeevonk server at the given address.
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let (reader, writer) = TcpStream::connect(addr).await?.into_split();
        let decoder = PacketDecoder::<ClientboundPacketPayload>::default();
        let encoder = PacketEncoder::default();
        let packet_reader = FramedRead::new(reader, decoder);
        let packet_writer = FramedWrite::new(writer, encoder);
        Ok(Self { packet_reader, packet_writer })
    }

    /// Requests the currently baked patch from the server.
    pub async fn request_patch(&mut self) -> io::Result<BakedPatch> {
        self.send_packet(ServerboundPacketPayload::RequestBakedPatch).await?;

        while let Some(packet) = self.packet_reader.next().await {
            match packet {
                Ok(packet) => match packet.payload {
                    ClientboundPacketPayload::ResponseBakedPatch(baked_patch) => {
                        return Ok(baked_patch);
                    }
                    _ => continue,
                },
                Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
            }
        }

        Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Connection closed"))
    }

    /// Requests the current DMX output (multiverse) from the server.
    pub async fn request_dmx_output(&mut self) -> io::Result<Multiverse> {
        self.send_packet(ServerboundPacketPayload::RequestDmxOutput).await?;

        while let Some(packet) = self.packet_reader.next().await {
            match packet {
                Ok(packet) => match packet.payload {
                    ClientboundPacketPayload::ResponseDmxOutput(multiverse) => {
                        return Ok(multiverse);
                    }
                    _ => continue,
                },
                Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
            }
        }

        Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Connection closed"))
    }

    /// Sets attribute values for fixtures on the server.
    pub async fn set_attribute_values(
        &mut self,
        values: Vec<(FixturePath, Attribute, ClampedValue)>,
    ) -> io::Result<()> {
        self.send_packet(ServerboundPacketPayload::RequestSetAttributeValues { values }).await?;

        while let Some(packet) = self.packet_reader.next().await {
            match packet {
                Ok(packet) => match packet.payload {
                    ClientboundPacketPayload::ResponseSetAttributeValues => {
                        return Ok(());
                    }
                    _ => continue,
                },
                Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
            }
        }

        Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Connection closed"))
    }

    /// Sends a packet with the given payload to the server.
    async fn send_packet(&mut self, payload: ServerboundPacketPayload) -> io::Result<()> {
        self.packet_writer
            .send(Packet::new(payload))
            .await
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }
}
