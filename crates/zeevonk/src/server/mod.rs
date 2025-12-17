//! The Zeevonk server serves as a hub to connect multiple clients
//! together and generating DMX output over various protocols.

use std::net::SocketAddr;
use std::sync::Arc;

use futures::{SinkExt as _, StreamExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{RwLock, RwLockReadGuard};
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::Error;
use crate::attr::Attribute;
use crate::dmx::Multiverse;
use crate::packet::{
    AttributeValues, ClientPacketPayload, Packet, PacketDecoder, PacketEncoder, ServerPacketPayload,
};
use crate::show::ShowData;
use crate::show::fixture::FixturePath;
use crate::showfile::Showfile;
use crate::value::ClampedValue;

mod protocols;
mod resolver;
mod show_data_builder;

pub struct Server<'sf> {
    showfile: &'sf Showfile,
    state: Arc<ServerState>,

    bound_addr: Option<SocketAddr>,
}

impl<'sf> Server<'sf> {
    pub fn new(showfile: &'sf Showfile) -> Result<Self, Error> {
        let state = Arc::new(ServerState::new(showfile)?);

        Ok(Self { showfile, state, bound_addr: None })
    }

    pub async fn start(&mut self) -> Result<(), Error> {
        log::info!("starting server...");

        let state = Arc::clone(&self.state);

        log::debug!("binding listener...");
        let address = self.showfile.config().address();
        let listener = TcpListener::bind(address).await?;
        self.bound_addr = Some(listener.local_addr().unwrap());
        log::debug!("listener bound");

        log::debug!("starting protocol manager");
        protocols::agent::start(self.showfile.protocols().clone(), Arc::clone(&state));
        log::debug!("protocol manager started");

        log::info!("zeevonk server started!");
        log::debug!("now accepting streams");
        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    let handler = ClientHandler::new(stream, peer, Arc::clone(&state));
                    tokio::spawn(async move { handler.run().await });
                }
                Err(e) => {
                    log::error!("accept error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Returns the address the socket has been bound to.
    ///
    /// # Panics
    ///
    /// Panics if the server has not been started yet.
    pub fn address(&self) -> SocketAddr {
        self.bound_addr.expect("server should have been started before calling this")
    }

    pub fn show_data(&'_ self) -> RwLockReadGuard<'_, ShowData> {
        self.state.show_data.blocking_read()
    }
}

#[derive(Debug)]
struct ServerState {
    show_data: RwLock<ShowData>,

    pending_attribute_values: RwLock<AttributeValues>,
    output_multiverse: RwLock<Multiverse>,
}

impl ServerState {
    pub fn new<'sf>(showfile: &'sf Showfile) -> Result<Self, Error> {
        let show_data = show_data_builder::build_from_showfile(showfile)?;

        Ok(Self {
            show_data: RwLock::new(show_data),

            pending_attribute_values: RwLock::new(AttributeValues::new()),
            output_multiverse: RwLock::new(Multiverse::new()),
        })
    }

    pub async fn process_packet(
        &self,
        packet: Packet<ServerPacketPayload>,
        peer: SocketAddr,
        writer: &mut FramedWrite<OwnedWriteHalf, PacketEncoder<ClientPacketPayload>>,
    ) {
        log::trace!("processing packet from {}", peer);

        let response = match packet.payload {
            ServerPacketPayload::RequestShowData => {
                let show_data = self.show_data.read().await.clone();
                Some(ClientPacketPayload::ResponseShowData(show_data))
            }
            ServerPacketPayload::RequestDmxOutput => {
                self.resolve_values().await;
                let multiverse = self.output_multiverse.read().await.clone();
                Some(ClientPacketPayload::ResponseDmxOutput(multiverse))
            }
            ServerPacketPayload::RequestSetAttributeValues(values) => {
                for ((fixture_path, attribute), value) in values.values() {
                    self.set_attribute_value(*fixture_path, *attribute, *value).await;
                }
                self.resolve_values().await;
                Some(ClientPacketPayload::ResponseSetAttributeValues)
            }
        };

        // If we have a response, send it back to the client.
        if let Some(payload) = response {
            let packet = Packet::new(payload);
            if let Err(e) = writer.send(packet).await {
                log::error!("failed to send response to {}: {}", peer, e);
            }
        }
    }

    async fn set_attribute_value(
        &self,
        fixture_path: FixturePath,
        attribute: Attribute,
        value: ClampedValue,
    ) {
        self.pending_attribute_values.write().await.set(fixture_path, attribute, value);
    }
}

struct ClientHandler {
    peer: SocketAddr,
    reader: FramedRead<OwnedReadHalf, PacketDecoder<ServerPacketPayload>>,
    writer: FramedWrite<OwnedWriteHalf, PacketEncoder<ClientPacketPayload>>,
    state: Arc<ServerState>,
}

impl ClientHandler {
    fn new(stream: TcpStream, peer: SocketAddr, state: Arc<ServerState>) -> Self {
        let (read_half, write_half) = stream.into_split();
        let decoder = PacketDecoder::<ServerPacketPayload>::default();
        let encoder = PacketEncoder::<ClientPacketPayload>::default();

        let framed_reader = FramedRead::new(read_half, decoder);
        let framed_writer = FramedWrite::new(write_half, encoder);

        Self { peer, reader: framed_reader, writer: framed_writer, state }
    }

    async fn run(mut self) {
        log::info!("client connected: {}", self.peer);

        while let Some(frame_res) = self.reader.next().await {
            match frame_res {
                Ok(packet) => {
                    self.state.process_packet(packet, self.peer, &mut self.writer).await;
                }
                Err(e) => {
                    log::error!("error reading packet from {}: {}", self.peer, e);
                    break;
                }
            }
        }

        log::info!("client disconnected: {}", self.peer);
    }
}
