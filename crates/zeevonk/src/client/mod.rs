//! A client that can communicate with a Zeevonk server (e.g. sending and receiving triggers or setting attribute values).

use std::sync::Arc;

use futures::{SinkExt, StreamExt as _};
use tokio::io;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::Mutex;
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::dmx::Multiverse;
use crate::packet::{
    AttributeValues, ClientPacketPayload, Packet, PacketDecoder, PacketEncoder, ServerPacketPayload,
};
use crate::state::State;

pub use processor::*;

mod processor;

pub struct Client {
    inner: Arc<Mutex<Inner>>,
}

impl Client {
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let (reader, writer) = TcpStream::connect(addr).await?.into_split();
        log::info!("client connected");

        let decoder = PacketDecoder::<ClientPacketPayload>::default();
        let encoder = PacketEncoder::<ServerPacketPayload>::default();
        let packet_reader = FramedRead::new(reader, decoder);
        let packet_writer = FramedWrite::new(writer, encoder);

        let inner = Arc::new(Mutex::new(Inner { packet_reader, packet_writer }));

        Ok(Self { inner })
    }

    pub async fn request_state(&self) -> io::Result<State> {
        let mut guard = self.inner.lock().await;
        guard.request_state().await
    }

    pub async fn request_dmx_output(&self) -> io::Result<Multiverse> {
        let mut guard = self.inner.lock().await;
        guard.request_dmx_output().await
    }

    pub async fn request_set_attribute_values(&self, values: AttributeValues) -> io::Result<()> {
        let mut guard = self.inner.lock().await;
        guard.request_set_attribute_values(values).await
    }
}

struct Inner {
    packet_reader: FramedRead<OwnedReadHalf, PacketDecoder<ClientPacketPayload>>,
    packet_writer: FramedWrite<OwnedWriteHalf, PacketEncoder<ServerPacketPayload>>,
}

impl Inner {
    pub async fn request_state(&mut self) -> io::Result<State> {
        self.send_packet(ServerPacketPayload::RequestState).await?;

        while let Some(packet) = self.packet_reader.next().await {
            match packet {
                Ok(packet) => match packet.payload {
                    ClientPacketPayload::ResponseState(state) => {
                        return Ok(state);
                    }
                    _ => continue,
                },
                Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
            }
        }

        Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Connection closed"))
    }

    pub async fn request_dmx_output(&mut self) -> io::Result<Multiverse> {
        self.send_packet(ServerPacketPayload::RequestDmxOutput).await?;

        while let Some(packet) = self.packet_reader.next().await {
            match packet {
                Ok(packet) => match packet.payload {
                    ClientPacketPayload::ResponseDmxOutput(multiverse) => {
                        return Ok(multiverse);
                    }
                    _ => continue,
                },
                Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
            }
        }

        Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Connection closed"))
    }

    pub async fn request_set_attribute_values(
        &mut self,
        values: AttributeValues,
    ) -> io::Result<()> {
        self.send_packet(ServerPacketPayload::RequestSetAttributeValues(values)).await?;

        while let Some(packet) = self.packet_reader.next().await {
            match packet {
                Ok(packet) => match packet.payload {
                    ClientPacketPayload::ResponseSetAttributeValues => {
                        return Ok(());
                    }
                    _ => continue,
                },
                Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
            }
        }

        Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Connection closed"))
    }

    async fn send_packet(&mut self, payload: ServerPacketPayload) -> io::Result<()> {
        self.packet_writer
            .send(Packet::new(payload))
            .await
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }
}
