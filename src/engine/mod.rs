use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

use anyhow::Context;
use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::dmx::Multiverse;
use crate::engine::output::DmxOutputManager;
use crate::gdcs::{self, GeneralizedDmxControlSystem};
use crate::packet::Packet;
use crate::packet::client::ClientboundPacketPayload;
use crate::packet::codec::{PacketDecoder, PacketEncoder};
use crate::packet::server::ServerboundPacketPayload;
use crate::showfile::Showfile;

/// DMX output handling.
mod output;

/// The `Engine` struct is responsible for managing the main runtime state of the application.
pub struct Engine<'sf> {
    showfile: &'sf Showfile,

    dmx_output_manager: DmxOutputManager,
    output_multiverse: Arc<RwLock<Multiverse>>,
    gdcs: Arc<RwLock<GeneralizedDmxControlSystem>>,

    /// Contains the listener after the engine has been started.
    listener: Option<TcpListener>,
}

impl<'sf> Engine<'sf> {
    /// Creates a new [Engine] for the given [Showfile].
    pub fn new(showfile: &'sf Showfile) -> Self {
        let output_multiverse = Arc::new(RwLock::new(Multiverse::new()));
        let gdcs = Arc::new(RwLock::new(GeneralizedDmxControlSystem::new()));

        Self {
            showfile,
            dmx_output_manager: DmxOutputManager::new(
                showfile.protocols(),
                output_multiverse.clone(),
            ),
            output_multiverse,
            gdcs,
            listener: None,
        }
    }

    /// Initializes and starts the engine.
    ///
    /// # Errors
    ///
    /// - The Tokio runtime fails to build.
    /// - The TCP listener fails to bind to the specified address.
    ///
    pub fn start(&mut self) -> anyhow::Result<()> {
        self.load_gdcs_gdtf_fixture_types()?;
        self.load_gdcs_fixtures()?;

        self.dmx_output_manager.start();

        // Start the Tokio Runtime.
        tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .build()
            .context("failed to build tokio runtime")?
            .block_on(async move {
                log::debug!("binding listener...");

                // Create a listener.
                let address = self.showfile.config().address();
                self.listener = Some(
                    TcpListener::bind(address)
                        .await
                        .with_context(|| format!("failed to bind the listener on {}", address))?,
                );

                log::debug!("accepting streams...");

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
    /// This function will panic if the engine has not been started yet.
    pub fn address(&self) -> SocketAddr {
        self.listener
            .as_ref()
            .expect("engine should have been started before calling this")
            .local_addr()
            .unwrap()
    }

    /// Handles an individual client connection.
    fn handle_client(&self, stream: TcpStream, socket_addr: SocketAddr) -> anyhow::Result<()> {
        log::info!("handling client");

        let (reader, writer) = stream.into_split();
        let mut framed_reader =
            FramedRead::new(reader, PacketDecoder::<ServerboundPacketPayload>::default());
        let mut framed_writer = FramedWrite::new(writer, PacketEncoder::default());

        let handle_packet = {
            let output_multiverse = Arc::clone(&self.output_multiverse);
            let gdcs = Arc::clone(&self.gdcs);
            async move |packet: Packet<ServerboundPacketPayload>,
                        socket_addr,
                        framed_writer: &mut FramedWrite<_, _>| {
                log::debug!("handling packet: {packet:?}");
                let response_payload = match packet.payload {
                    ServerboundPacketPayload::RequestLayout => {
                        let gdcs_fixtures =
                            gdcs.read().unwrap().fixtures().into_iter().cloned().collect();
                        let layout = Layout { fixtures: gdcs_fixtures };

                        Some(ClientboundPacketPayload::ResponseLayout(layout))
                    }
                    ServerboundPacketPayload::RequestDmxOutput => {
                        gdcs.write().unwrap().resolve();
                        *output_multiverse.write().unwrap() =
                            gdcs.read().unwrap().resolved_multiverse().clone();
                        let multiverse = output_multiverse.read().unwrap().clone();
                        Some(ClientboundPacketPayload::ResponseDmxOutput(multiverse))
                    }
                    ServerboundPacketPayload::RequestTriggers => {
                        Some(ClientboundPacketPayload::ResponseTriggers)
                    }
                    ServerboundPacketPayload::RequestAttributeValues => {
                        Some(ClientboundPacketPayload::ResponseAttributeValues)
                    }
                    ServerboundPacketPayload::RequestSetAttributeValues { values } => {
                        for (fixture_path, attribute, value) in values {
                            gdcs.write().unwrap().set_channel_function_value(
                                fixture_path,
                                attribute,
                                value,
                            );
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

    fn load_gdcs_gdtf_fixture_types(&mut self) -> anyhow::Result<()> {
        for gdtf_file_path in self.showfile.gdtf_file_paths() {
            self.gdcs.write().unwrap().register_gdtf_file(gdtf_file_path)?;
        }

        Ok(())
    }

    fn load_gdcs_fixtures(&mut self) -> anyhow::Result<()> {
        for fixture in self.showfile.patch().fixtures() {
            self.gdcs.write().unwrap().register_fixture(
                fixture.id(),
                fixture.label().to_string(),
                fixture.address(),
                fixture.kind().gdtf_fixture_type_id(),
                fixture.kind().gdtf_dmx_mode().to_string(),
            )?;
        }

        Ok(())
    }
}

/// Contains the complete layout of the patch, (sub)fixtures and their
/// channel functions.
#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Layout {
    fixtures: Vec<gdcs::fixture::Fixture>,
}

impl Layout {
    /// Gets all (sub)fixtures.
    pub fn fixtures(&self) -> &[gdcs::fixture::Fixture] {
        &self.fixtures
    }
}
