//! A processor client can tell the server what attributes have to be updated.
//!
//! **Note:** The `client-processor` feature must be enabled to use a processor client in your code.
//!
//! A processor client is responsible for generating high-level [GDTF](https://gdtf.eu) attribute values for
//! specific fixtures and sending them to the server.
//!
//! Typical responsibilities of a processor client include:
//! - Subscribing to [triggers](crate::trigger).
//! - Mapping [triggers](crate::trigger) to fixture/attribute targets and resolving which attributes should change.
//! - Calculating or interpolating attribute values (effects, fades, curves, color mixing, etc.).
//! - Sending attribute updates to the server for DMX output.
//! - Maintaining local state and managing transitions (so updates are smooth and deterministic).
//!
//! # Examples
//!
//! FIXME: Add examples.

use tokio::sync::mpsc;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::packet::processor::ServerboundPacket;
use crate::value::AttributeValues;

/// A processor client can tell the server what attributes have to be updated.
pub struct Client {
    outbound_tx: mpsc::UnboundedSender<ServerboundPacket>,
    outbound_rx: Option<mpsc::UnboundedReceiver<ServerboundPacket>>,
}
impl Client {
    /// Create a new processor client.
    pub fn new() -> Self {
        let (outbound_tx, outbound_rx) = mpsc::unbounded_channel::<ServerboundPacket>();
        Self { outbound_tx, outbound_rx: Some(outbound_rx) }
    }

    /// Connect to a processor server at the given WebSocket `uri`.
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

    /// Ask the server to update attributes.
    pub async fn update_attributes(
        &self,
        values: AttributeValues,
        include_children: bool,
    ) -> crate::client::Result<()> {
        self.outbound_tx
            .send(ServerboundPacket::UpdateAttributes { values, include_children })
            .map_err(|_| crate::client::Error::ConnectionClosed)?;
        Ok(())
    }
}
