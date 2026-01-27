//! A client that connects to a Zeevonk server and can act as both a controller and a processor.
//!
//! **Note:** The `client` feature must be enabled to use this module.
//!
//! As a controller, the client can send triggers (for example, button presses or fader moves)
//! to the server so they can be routed to other clients. As a processor, the client can send
//! fixture attribute updates (GDTF attributes like color, position, intensity) to the server
//! for DMX output and can receive triggers from other clients.
//!
//! Controller-style example (sending triggers):
//!
//! ```ignore
//! use zeevonk::client::Client;
//! use zeevonk::ident::Identifier;
//! use zeevonk::trigger::{Trigger, TriggerValue};
//!
//! #[tokio::main]
//! async fn main() {
//!     pretty_env_logger::init();
//!
//!     let mut client = Client::new(Identifier::new("zv-example-controller").unwrap());
//!     client.connect("ws://127.0.0.1:7334").await.unwrap();
//!
//!     loop {
//!         client
//!             .send_trigger(Trigger::new(
//!                 Identifier::new("button-1").unwrap(),
//!                 TriggerValue::Boolean(true),
//!             ))
//!             .await
//!             .unwrap();
//!
//!         std::thread::sleep(std::time::Duration::from_secs_f32(1.0 / 10.0));
//!     }
//! }
//! ```
//!
//! Processor-style example (sending attribute updates and receiving triggers):
//!
//! ```ignore
//! use zeevonk::attr::Attribute;
//! use zeevonk::client::Client;
//! use zeevonk::ident::Identifier;
//! use zeevonk::project::stage::{FixtureId, FixtureIdPart};
//! use zeevonk::value::AttributeValues;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut client = Client::new(Identifier::new("zv-example-processor").unwrap());
//!     client.on_trigger(|from_client, trigger| eprintln!("{from_client}: {trigger:?}"));
//!     client.connect("ws://127.0.0.1:7334").await.unwrap();
//!
//!     let fid = FixtureId::new(FixtureIdPart::new(101).unwrap());
//!
//!     let mut values = AttributeValues::new();
//!
//!     // Set attribute values for your fixtures here...
//!     values.set(fid, Attribute::Dimmer, 1.0);
//!
//!     client.update_attributes(values, false).await.unwrap();
//!
//!     loop {}
//! }
//! ```
//!
//! For advanced usage you can maintain local state and manage transitions for smooth updates.

pub mod error;

use std::sync::Arc;

pub use error::{Error, Result};

use futures_util::lock::Mutex;
use tokio::sync::{mpsc, oneshot};

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::ident::Identifier;
use crate::packet::{ClientboundPacket, ServerboundPacket};
use crate::project::Project;
use crate::trigger::Trigger;
use crate::value::AttributeValues;

/// A client that connects to the server, sends and receives packets
/// and handles triggers.
pub struct Client {
    client_id: Identifier,

    outbound_tx: mpsc::UnboundedSender<ServerboundPacket>,
    outbound_rx: Option<mpsc::UnboundedReceiver<ServerboundPacket>>,

    on_trigger: Option<Arc<dyn Fn(Identifier, Trigger) + Send + Sync>>,

    // Used to deliver project data responses for RequestProjectData packet.
    project_request: Arc<Mutex<Option<oneshot::Sender<Project>>>>,
}

impl Client {
    /// Create a new client.
    pub fn new(client_id: Identifier) -> Self {
        let (outbound_tx, outbound_rx) = mpsc::unbounded_channel::<ServerboundPacket>();

        Self {
            client_id,
            outbound_tx,
            outbound_rx: Some(outbound_rx),
            on_trigger: None,
            project_request: Arc::new(Mutex::new(None)),
        }
    }

    /// This client's identifier. It's used to identify this client on the server so
    /// triggers can be routed from and to specific clients in the project config.
    pub fn client_id(&self) -> &Identifier {
        &self.client_id
    }

    /// Set a callback to be called when a trigger is received from the server.
    ///
    /// The callback receives the source client [`Identifier`] and the [`Trigger`] itself.
    pub fn on_trigger<F: Fn(Identifier, Trigger) + Send + Sync + 'static>(&mut self, f: F) {
        self.on_trigger = Some(Arc::new(f));
    }

    /// Set a callback to be called when a trigger is received from the server.
    ///
    /// The callback receives the source client [`Identifier`] and the [`Trigger`] itself.
    pub fn with_on_trigger<F: Fn(Identifier, Trigger) + Send + Sync + 'static>(
        mut self,
        f: F,
    ) -> Self {
        self.on_trigger = Some(Arc::new(f));
        self
    }

    /// Connect to a server at the given WebSocket `uri`.
    ///
    /// If the client is already connected this method returns immediately with `Ok(())`.
    pub async fn connect(&mut self, uri: &str) -> crate::client::Result<()> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(uri)
            .await
            .map_err(|_| crate::client::Error::ServerConnectionFailed { uri: uri.to_string() })?;
        let (mut message_write, mut message_read) = ws_stream.split();

        let Some(mut outbound_rx) = self.outbound_rx.take() else {
            // Client already connected.
            return Ok(());
        };

        let on_trigger = self.on_trigger.clone();
        let project_request = self.project_request.clone();

        self.send_packet(ServerboundPacket::Register { client_id: self.client_id.clone() }).await?;

        let handle_packet = {
            let on_trigger = on_trigger.clone();
            let project_request = project_request.clone();
            async move |packet: ClientboundPacket| {
                match packet {
                    ClientboundPacket::RegisterSuccess => {
                        log::info!("client sucessfully registered at the server");
                    }

                    ClientboundPacket::Trigger { from_client_id, trigger } => {
                        log::debug!("received trigger from {}: {:?}", from_client_id, trigger);
                        if let Some(cb) = &on_trigger {
                            (cb)(from_client_id, trigger);
                        }
                    }

                    ClientboundPacket::ProjectData { project } => {
                        log::debug!("received project data");
                        // If there's a waiter for project data, send it the project.
                        let mut guard = project_request.lock().await;
                        if let Some(tx) = guard.take() {
                            if tx.send(project).is_err() {
                                log::warn!(
                                    "project data receiver dropped before response could be sent"
                                );
                            }
                        } else {
                            log::debug!("no active waiter for project data");
                        }
                    }
                }
            }
        };

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    outbound = outbound_rx.recv() => {
                        match outbound {
                            Some(packet) => {
                                let message = match serde_json::to_string(&packet) {
                                    Ok(json) => Message::Text(json.into()),
                                    Err(e) => {
                                        log::error!("failed to serialize packet: {e}");
                                        // Could not serialize, skip this packet.
                                        continue;
                                    }
                                };
                                if let Err(e) = message_write.send(message).await {
                                    log::info!("connection with the server has been closed (outbound): {e}");
                                    break;
                                }
                            }
                            None => {
                                log::info!("outbound channel closed");
                                break;
                            }
                        }
                    }
                    inbound = message_read.next() => {
                        match inbound {
                            Some(Ok(Message::Text(text))) => {
                                match serde_json::from_str::<ClientboundPacket>(&text) {
                                    Ok(packet) => {
                                        handle_packet(packet).await;
                                    }
                                    Err(e) => {
                                        log::warn!("failed to parse packet: {e} (raw: {text:?})");
                                    }
                                }
                            }
                            Some(Ok(Message::Close(_))) | None => {
                                log::info!("connection with the server has been closed (inbound)");
                                break;
                            }
                            Some(Ok(_)) => {}
                            Some(Err(e)) => {
                                log::warn!("WebSocket error: {e}");
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Send a [`Trigger`] to the server so it can route it to the correct clients.
    pub async fn send_trigger(&self, trigger: Trigger) -> crate::client::Result<()> {
        self.send_packet(ServerboundPacket::Trigger { trigger }).await?;
        Ok(())
    }

    /// Let the server know about new attribute values it should update the DMX output for.
    ///
    /// `include_children` Indicates whether updates propagate attribute values to child fixtures.
    pub async fn update_attributes(
        &self,
        values: AttributeValues,
        include_children: bool,
    ) -> crate::client::Result<()> {
        self.send_packet(ServerboundPacket::UpdateAttributes { values, include_children }).await?;
        Ok(())
    }

    /// Request the full project data from the server.
    pub async fn project_data(&self) -> crate::client::Result<Project> {
        let (tx, rx) = oneshot::channel::<Project>();
        *self.project_request.lock().await = Some(tx);

        self.send_packet(ServerboundPacket::RequestProjectData).await?;

        match rx.await {
            Ok(project) => Ok(project),
            Err(_) => Err(crate::client::Error::ConnectionClosed),
        }
    }

    async fn send_packet(&self, packet: ServerboundPacket) -> crate::client::Result<()> {
        self.outbound_tx.send(packet).map_err(|_| crate::client::Error::ConnectionClosed)?;
        Ok(())
    }
}
