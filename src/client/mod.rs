use std::sync::Arc;

use futures::{SinkExt, StreamExt as _};
use tokio::io;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::Mutex;
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::core::dmx::Multiverse;
use crate::core::packet::{
    ClientboundPacketPayload, Packet, PacketDecoder, PacketEncoder, ServerboundPacketPayload,
};
use crate::server::{AttributeValues, BakedPatch};

pub use processor::*;

mod processor;

/// The Zeevonk client.
pub struct Client {
    inner: Arc<Mutex<Inner>>,
}

impl Client {
    /// Connects to a Zeevonk server at the given address.
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let (reader, writer) = TcpStream::connect(addr).await?.into_split();
        let decoder = PacketDecoder::<ClientboundPacketPayload>::default();
        let encoder = PacketEncoder::default();
        let packet_reader = FramedRead::new(reader, decoder);
        let packet_writer = FramedWrite::new(writer, encoder);

        let inner = Arc::new(Mutex::new(Inner { packet_reader, packet_writer }));

        Ok(Self { inner })
    }

    /// Requests the currently baked patch from the server.
    pub async fn request_patch(&self) -> io::Result<BakedPatch> {
        let mut guard = self.inner.lock().await;
        guard.request_patch().await
    }

    /// Requests the current DMX output (multiverse) from the server.
    pub async fn request_dmx_output(&self) -> io::Result<Multiverse> {
        let mut guard = self.inner.lock().await;
        guard.request_dmx_output().await
    }

    /// Requests setting attribute values for fixtures on the server.
    pub async fn request_set_attribute_values(&self, values: AttributeValues) -> io::Result<()> {
        let mut guard = self.inner.lock().await;
        guard.request_set_attribute_values(values).await
    }
}

struct Inner {
    packet_reader: FramedRead<OwnedReadHalf, PacketDecoder<ClientboundPacketPayload>>,
    packet_writer: FramedWrite<OwnedWriteHalf, PacketEncoder<ServerboundPacketPayload>>,
}

impl Inner {
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

    /// Requests setting attribute values for fixtures on the server.
    pub async fn request_set_attribute_values(
        &mut self,
        values: AttributeValues,
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
