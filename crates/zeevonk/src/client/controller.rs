//! The controller client sends triggers (like button presses or fader moves) to the Zeevonk server.//!
//! **Note:** The `client-controller` feature must be enabled to use a controller client in your code.
//!
//! You can write software to, for example, receive MIDI or OSC messages from a control surface,
//! and then use the controller client to send triggers to the Zeevonk server.
//! This allows you to connect your own hardware or software controls such as MIDI controllers,
//! OSC apps, or custom UIs to Zeevonk by translating their events into triggers
//! that the server can route to controller clients.
//!
//! ## Example
//!
//! ```ignore
//! use zeevonk::client::controller::Client;
//! use zeevonk::ident::Identifier;
//! use zeevonk::trigger::{Trigger, TriggerValue};
//!
//! #[tokio::main]
//! async fn main() {
//!     pretty_env_logger::init();
//!
//!     let mut client = Client::new(Identifier::new("zv-example-controller").unwrap());
//!     client.connect("ws://127.0.0.1:7335").await.unwrap();
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
use tokio::sync::mpsc;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::ident::Identifier;
use crate::packet::controller::{ClientboundPacket, ServerboundPacket};
use crate::trigger::Trigger;

/// A controller client can tell the server what attributes have to be updated.
pub struct Client {
    client_id: Identifier,

    outbound_tx: mpsc::UnboundedSender<ServerboundPacket>,
    outbound_rx: Option<mpsc::UnboundedReceiver<ServerboundPacket>>,
}
impl Client {
    /// Create a new controller client.
    pub fn new(client_id: Identifier) -> Self {
        let (outbound_tx, outbound_rx) = mpsc::unbounded_channel::<ServerboundPacket>();
        Self { client_id, outbound_tx, outbound_rx: Some(outbound_rx) }
    }

    /// This client's identifier. It's used to identify this client on the server so
    /// triggers can be routed from and to specific clients in the project config.
    pub fn client_id(&self) -> &Identifier {
        &self.client_id
    }

    /// Connect to a controller server at the given WebSocket `uri`.
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

        self.send_packet(ServerboundPacket::Register { client_id: self.client_id.clone() }).await?;

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
                                        log::debug!("received packet: {:?}", packet);
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

    /// Send a [`Trigger`] to the server so it can route it to the correct
    /// processor clients.
    pub async fn send_trigger(&self, trigger: Trigger) -> crate::client::Result<()> {
        self.send_packet(ServerboundPacket::Trigger { trigger }).await?;
        Ok(())
    }

    async fn send_packet(&self, packet: ServerboundPacket) -> crate::client::Result<()> {
        self.outbound_tx.send(packet).map_err(|_| crate::client::Error::ConnectionClosed)?;
        Ok(())
    }
}
