use std::io::{self};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

use crate::server;
use crate::server::state::State;

pub mod controller;
pub mod processor;

#[async_trait::async_trait]
pub trait PacketHandler: Send + Sync + 'static {
    type ServerPacket: serde::de::DeserializeOwned + Send + 'static;
    type ClientPacket: serde::Serialize + Send + 'static;

    async fn on_disconnect(address: SocketAddr, state: Arc<State>);

    async fn handle_packet(
        packet: Self::ServerPacket,
        address: SocketAddr,
        writer: &mpsc::Sender<Self::ClientPacket>,
        state: Arc<State>,
    ) -> Result<(), server::Error>;
}

pub async fn start_ws_listener<H: PacketHandler>(
    port: u16,
    label: &'static str,
    state: Arc<State>,
) -> Result<(), server::Error> {
    let address = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port));

    let listener = TcpListener::bind(&address).await.map_err(|e| {
        log::error!("failed to bind {} listener to {}: {}", label, address, e);
        server::Error::from(e)
    })?;

    log::info!("{} listener active on {}", label, address);

    loop {
        let (stream, addr) = listener.accept().await.map_err(|e| {
            log::error!("{} accept error: {}", label, e);
            server::Error::from(e)
        })?;

        let state = Arc::clone(&state);
        tokio::spawn(async move {
            if let Err(e) = handle_ws_client::<H>(stream, addr, state).await {
                log::error!("{} client {} failed: {}", label, addr, e);
            }
        });
    }
}

async fn handle_ws_client<H: PacketHandler>(
    stream: TcpStream,
    address: SocketAddr,
    state: Arc<State>,
) -> Result<(), server::Error> {
    let ws_stream =
        accept_async(stream).await.map_err(|e| server::Error::from(io::Error::other(e)))?;

    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = mpsc::channel::<H::ClientPacket>(16);

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

    while let Some(msg) = read.next().await {
        let packet = match msg {
            Ok(Message::Text(text)) => serde_json::from_str::<H::ServerPacket>(&text).ok(),
            Ok(Message::Binary(bin)) => serde_json::from_slice::<H::ServerPacket>(&bin).ok(),
            Ok(Message::Close(_)) => break,
            _ => None,
        };

        if let Some(packet) = packet {
            H::handle_packet(packet, address, &tx, Arc::clone(&state)).await?;
        }
    }

    H::on_disconnect(address, Arc::clone(&state)).await;
    drop(tx);
    let _ = writer_task.await;

    Ok(())
}
