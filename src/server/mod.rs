use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::thread;

use anyhow::Context;
use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::core::dmx::Multiverse;
use crate::core::gdcs::{self, Attribute, ClampedValue, FixturePath, GeneralizedDmxControlSystem};
use crate::core::packet::{
    ClientboundPacketPayload, Packet, PacketDecoder, PacketEncoder, ServerboundPacketPayload,
};
use crate::core::showfile::Showfile;
use crate::server::output::DmxOutputManager;

mod output;
mod sacn;

/// The Zeevonk server.
pub struct Server<'sf> {
    showfile: &'sf Showfile,

    output_multiverse: Arc<RwLock<Multiverse>>,
    gdcs: Arc<RwLock<GeneralizedDmxControlSystem>>,

    /// Contains the listener after the server has been started.
    listener: Option<TcpListener>,
}

impl<'sf> Server<'sf> {
    /// Creates a new [Server] for the given [Showfile].
    pub fn new(showfile: &'sf Showfile) -> Self {
        let output_multiverse = Arc::new(RwLock::new(Multiverse::new()));
        let gdcs = Arc::new(RwLock::new(GeneralizedDmxControlSystem::new()));

        Self { showfile, output_multiverse, gdcs, listener: None }
    }

    /// Initializes and starts the server.
    pub fn start(&mut self) -> anyhow::Result<()> {
        log::info!("starting server");

        self.start_dmx_output_manager()?;

        self.gdcs.write().unwrap().insert_showfile_data(self.showfile)?;

        // Start the Tokio Runtime.
        tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .build()
            .context("failed to build tokio runtime")?
            .block_on(async move {
                log::debug!("binding listener");

                // Create a listener.
                let address = self.showfile.config().address();
                self.listener = Some(
                    TcpListener::bind(address)
                        .await
                        .with_context(|| format!("failed to bind the listener on {}", address))?,
                );

                log::debug!("accepting streams");

                // For each new incoming connection, run the handle_client function.
                while let Ok((stream, socket_addr)) = self
                    .listener
                    .as_mut()
                    .expect("listener should have just been set")
                    .accept()
                    .await
                {
                    // Let's just log the error if a client handler fails.
                    if let Err(err) =
                        self.handle_client(stream, socket_addr).context("client handler")
                    {
                        log::error!("error handling client {}: {}", socket_addr, err);
                    };
                }

                Ok::<_, anyhow::Error>(())
            })
            .context("top level future")?;

        Ok(())
    }

    /// Returns the address the socket has been bound to.
    /// Note that this could be different from the address set in
    /// the showfile config, as using port 0 in the config will return the
    /// actually provided port in this [SocketAddr].
    ///
    /// # Panics
    ///
    /// This function will panic if the server has not been started yet.
    pub fn address(&self) -> SocketAddr {
        self.listener
            .as_ref()
            .expect("server should have been started before calling this")
            .local_addr()
            .unwrap()
    }

    fn start_dmx_output_manager(&mut self) -> anyhow::Result<()> {
        let mut dmx_output_manager =
            DmxOutputManager::new(self.showfile.protocols(), self.output_multiverse.clone())?;

        // FIXME: We should make sure this thread closes if we drop the server.
        thread::spawn(move || {
            if let Err(err) = dmx_output_manager.start() {
                log::error!("failed to start DMX output manager: {err}");
            };
        });

        Ok(())
    }

    /// Handles an individual client connection.
    fn handle_client(&self, stream: TcpStream, socket_addr: SocketAddr) -> anyhow::Result<()> {
        log::info!("handling client");

        let (reader, writer) = stream.into_split();
        let mut framed_reader =
            FramedRead::new(reader, PacketDecoder::<ServerboundPacketPayload>::default());
        let mut framed_writer =
            FramedWrite::new(writer, PacketEncoder::<ClientboundPacketPayload>::default());

        let handle_packet = {
            let output_multiverse = Arc::clone(&self.output_multiverse);
            let gdcs = Arc::clone(&self.gdcs);
            async move |packet: Packet<ServerboundPacketPayload>,
                        socket_addr,
                        framed_writer: &mut FramedWrite<_, _>| {
                log::trace!("handling incoming packet");
                let response_payload = match packet.payload {
                    ServerboundPacketPayload::RequestBakedPatch => {
                        let gdcs_fixtures =
                            gdcs.read().unwrap().fixtures().into_iter().cloned().collect();
                        let baked_patch = BakedPatch { fixtures: gdcs_fixtures };

                        Some(ClientboundPacketPayload::ResponseBakedPatch(baked_patch))
                    }
                    ServerboundPacketPayload::RequestDmxOutput => {
                        gdcs.write().unwrap().resolve();
                        *output_multiverse.write().unwrap() =
                            gdcs.read().unwrap().resolved_multiverse().clone();
                        let multiverse = output_multiverse.read().unwrap().clone();
                        Some(ClientboundPacketPayload::ResponseDmxOutput(multiverse))
                    }
                    ServerboundPacketPayload::RequestSetAttributeValues { values } => {
                        for ((fixture_path, attribute), value) in values.values {
                            let mut gdcs_lock = gdcs.write().unwrap();
                            gdcs_lock.set_channel_function_value(fixture_path, attribute, value);
                            gdcs_lock.resolve();
                            *output_multiverse.write().unwrap() =
                                gdcs_lock.resolved_multiverse().clone();
                        }

                        Some(ClientboundPacketPayload::ResponseSetAttributeValues)
                    }
                };

                if let Some(response_payload) = response_payload {
                    let packet = Packet::new(response_payload);
                    if let Err(err) = framed_writer.send(packet).await {
                        log::error!("error sending response to {}: {}", socket_addr, err);
                    }
                }
            }
        };

        tokio::spawn(async move {
            while let Some(packet) = framed_reader.next().await {
                match packet {
                    Ok(packet) => {
                        handle_packet(packet, socket_addr, &mut framed_writer).await;
                    }
                    Err(err) => {
                        log::error!("error reading packet from {}: {}", socket_addr, err);
                        break;
                    }
                }
            }
        });

        Ok(())
    }
}

/// Stores attribute values for fixtures, mapping each (FixturePath, Attribute)
/// pair to its corresponding [ClampedValue].
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct AttributeValues {
    values: HashMap<(FixturePath, Attribute), ClampedValue>,
}

impl AttributeValues {
    /// Creates a new [AttributeValues].
    pub fn new() -> Self {
        Self { values: HashMap::new() }
    }

    /// Sets the value for a given fixture path and attribute.
    pub fn set(
        &mut self,
        fixture_path: FixturePath,
        attribute: Attribute,
        value: impl Into<ClampedValue>,
    ) {
        self.values.insert((fixture_path, attribute), value.into());
    }
}

/// Contains the complete baked patch, containing (sub)fixtures
/// and their channel functions.
#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct BakedPatch {
    fixtures: Vec<gdcs::Fixture>,
}

impl BakedPatch {
    /// Gets all (sub)fixtures.
    pub fn fixtures(&self) -> &[gdcs::Fixture] {
        &self.fixtures
    }
}
