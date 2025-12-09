use std::net::SocketAddr;

use anyhow::Context;
use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::engine::output::DmxOutputManager;
use crate::packet::{self, ClientboundPacket, ServerboundPacket};
use crate::showfile::Showfile;

mod output;

pub struct Engine<'sf> {
    showfile: &'sf Showfile,

    dmx_output_manager: DmxOutputManager,

    /// Contains the listener after the engine has been started.
    listener: Option<TcpListener>,
}

impl<'sf> Engine<'sf> {
    pub fn new(showfile: &'sf Showfile) -> Self {
        Self {
            showfile,
            dmx_output_manager: DmxOutputManager::new(showfile.protocols()),
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

    fn handle_client(&self, stream: TcpStream, socket_addr: SocketAddr) -> anyhow::Result<()> {
        log::info!("handling client");

        let (reader, writer) = stream.into_split();
        let mut framed_reader = FramedRead::new(reader, packet::codec::ServerboundPacketDecoder);
        let mut framed_writer = FramedWrite::new(writer, packet::codec::ClientboundPacketEncoder);

        let handle_packet = async |packet, socket_addr, framed_writer: &mut FramedWrite<_, _>| {
            let response = match packet {
                ServerboundPacket::RequestLayout => Some(ClientboundPacket::ResponseLayout),
                ServerboundPacket::RequestDmxOutput => Some(ClientboundPacket::ResponseDmxOutput),
                ServerboundPacket::RequestTriggers => Some(ClientboundPacket::ResponseTriggers),
                ServerboundPacket::RequestAttributeValues => {
                    Some(ClientboundPacket::ResponseAttributeValues)
                }
                ServerboundPacket::RequestSetAttributeValues => {
                    Some(ClientboundPacket::ResponseSetAttributeValues)
                }
            };

            if let Some(resp) = response {
                if let Err(err) = framed_writer.send(resp).await {
                    log::error!("error sending response to {}: {}", socket_addr, err);
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
