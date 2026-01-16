#[cfg(feature = "client-controller")]
pub mod controller;
#[cfg(feature = "client-processor")]
pub mod processor;

mod error;

pub use error::Error;

use std::io;
use std::net::SocketAddr;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::{Identifier, client};

#[async_trait::async_trait]
trait ClientPacketHandler: Send + Sync + 'static {
    type ServerPacket: serde::Serialize + Send + 'static;
    type ClientPacket: serde::de::DeserializeOwned + Send + 'static;

    async fn on_connect(
        id: Identifier,
        writer: &mpsc::Sender<Self::ServerPacket>,
    ) -> Result<(), client::Error>;

    async fn on_disconnect();

    async fn handle_packet(
        packet: Self::ClientPacket,
        writer: &mpsc::Sender<Self::ServerPacket>,
    ) -> Result<(), client::Error>;
}

async fn start_ws_client<H: ClientPacketHandler>(
    id: Identifier,
    server_addr: SocketAddr,
) -> Result<ClientHandle<H::ServerPacket>, client::Error> {
    let url = format!("ws://{}", server_addr);

    log::info!("{} client connecting to {}", id, url);

    let (ws_stream, _) =
        connect_async(&url).await.map_err(|e| client::Error::from(io::Error::other(e)))?;

    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = mpsc::channel::<H::ServerPacket>(16);

    let handle = ClientHandle { sender: tx.clone() };

    let writer_task = tokio::spawn(async move {
        while let Some(packet) = rx.recv().await {
            match serde_json::to_string(&packet) {
                Ok(json) => {
                    if write.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                Err(_) => {
                    let _ = write.send(Message::Close(None)).await;
                    break;
                }
            }
        }
        let _ = write.close().await;
    });

    H::on_connect(id, &tx).await?;

    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            let packet = match msg {
                Ok(Message::Text(text)) => serde_json::from_str::<H::ClientPacket>(&text).ok(),
                Ok(Message::Binary(bin)) => serde_json::from_slice::<H::ClientPacket>(&bin).ok(),
                Ok(Message::Close(_)) => break,
                _ => None,
            };

            if let Some(packet) = packet {
                if let Err(e) = H::handle_packet(packet, &tx).await {
                    log::error!("client packet handler error: {}", e);
                    break;
                }
            }
        }

        H::on_disconnect().await;
        drop(tx);
        let _ = writer_task.await;
    });

    Ok(handle)
}

pub struct ClientHandle<P> {
    sender: mpsc::Sender<P>,
}

impl<P> ClientHandle<P> {
    pub async fn send(&self, packet: P) -> Result<(), mpsc::error::SendError<P>> {
        self.sender.send(packet).await
    }
}
