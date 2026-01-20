//! The processor client can send fixture attribute values to the Zeevonk server.
//!
//! **Note:** The `client-processor` feature must be enabled to use a processor client in your code.
//!
//! Use the processor client to generate and update lighting [GDTF](https://gdtf.eu) attributes
//! (like color, position, or intensity) for fixtures, and send them to the server for DMX output.
//!
//! ## Example
//!
//! ```ignore
//! use zeevonk::attr::Attribute;
//! use zeevonk::client::processor::Client;
//! use zeevonk::ident::Identifier;
//! use zeevonk::project::patch::{FixtureId, FixtureIdPart};
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
//! The processor client is typically used to:
//! - Subscribe to [triggers](crate::trigger)
//! - Map triggers to fixture attributes
//! - Calculate or interpolate attribute values (effects, fades, color mixing, etc.)
//! - Send updates to the server for DMX output
//!
//! For advanced usage, you can maintain local state and manage transitions for smooth updates.

use std::sync::Arc;
use tokio::sync::mpsc;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::ident::Identifier;
use crate::packet::processor::{ClientboundPacket, ServerboundPacket};
use crate::trigger::Trigger;
use crate::value::AttributeValues;

/// A processor client can tell the server what attributes have to be updated.
pub struct Client {
    client_id: Identifier,

    outbound_tx: mpsc::UnboundedSender<ServerboundPacket>,
    outbound_rx: Option<mpsc::UnboundedReceiver<ServerboundPacket>>,

    on_trigger: Option<Arc<dyn Fn(Identifier, Trigger) + Send + Sync>>,
}

impl Client {
    /// Create a new processor client.
    pub fn new(client_id: Identifier) -> Self {
        let (outbound_tx, outbound_rx) = mpsc::unbounded_channel::<ServerboundPacket>();
        Self { client_id, outbound_tx, outbound_rx: Some(outbound_rx), on_trigger: None }
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

    /// Connect to a Zeevonk server at the given WebSocket `uri`.
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

        self.send_packet(ServerboundPacket::Register { client_id: self.client_id.clone() }).await?;

        let handle_packet = move |packet| match packet {
            ClientboundPacket::Trigger { from_client_id, trigger } => {
                log::debug!("received trigger from {}: {:?}", from_client_id, trigger);
                if let Some(cb) = &on_trigger {
                    (cb)(from_client_id, trigger);
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
                                        handle_packet(packet);
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

    async fn send_packet(&self, packet: ServerboundPacket) -> crate::client::Result<()> {
        self.outbound_tx.send(packet).map_err(|_| crate::client::Error::ConnectionClosed)?;
        Ok(())
    }
}
