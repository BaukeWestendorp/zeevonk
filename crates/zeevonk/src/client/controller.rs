//! The controller client sends triggers (like button presses or fader moves) to the Zeevonk server.
//!
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
//! ```
//! use zeevonk::client::controller::Client;
//! use zeevonk::ident::Identifier;
//! use zeevonk::trigger::{Trigger, TriggerValue};
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut client = Client::new();
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

use crate::packet::controller::ServerboundPacket;
use crate::trigger::Trigger;

/// A controller client can tell the server what attributes have to be updated.
pub struct Client {
    outbound_tx: mpsc::UnboundedSender<ServerboundPacket>,
    outbound_rx: Option<mpsc::UnboundedReceiver<ServerboundPacket>>,
}
impl Client {
    /// Create a new controller client.
    pub fn new() -> Self {
        let (outbound_tx, outbound_rx) = mpsc::unbounded_channel::<ServerboundPacket>();
        Self { outbound_tx, outbound_rx: Some(outbound_rx) }
    }

    /// Connect to a controller server at the given WebSocket `uri`.
    ///
    /// If the client is already connected this method returns immediately with `Ok(())`.
    pub async fn connect(&mut self, uri: &str) -> crate::Result<()> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(uri).await.unwrap();
        let (mut message_write, _message_read) = ws_stream.split();

        let Some(mut outbound_rx) = self.outbound_rx.take() else {
            // Client already connected.
            return Ok(());
        };

        tokio::spawn(async move {
            while let Some(packet) = outbound_rx.recv().await {
                let message = Message::Text(serde_json::to_string(&packet).unwrap().into());

                if message_write.send(message).await.is_err() {
                    log::info!("connection with the server has been closed");
                    break;
                }
            }
        });

        Ok(())
    }

    /// Send a [`Trigger`] to the server so it can route it to the correct
    /// processor clients.
    pub async fn send_trigger(&self, trigger: Trigger) -> crate::client::Result<()> {
        self.outbound_tx
            .send(ServerboundPacket::Trigger { trigger })
            .map_err(|_| crate::client::Error::ConnectionClosed)?;
        Ok(())
    }
}
