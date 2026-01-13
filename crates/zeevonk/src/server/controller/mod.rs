use std::io;
use std::net::SocketAddr;

use futures::StreamExt;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message as WsMessage;

use crate::server::controller::packet::ServerControllerPacket;
use crate::server::{self};

pub mod packet;

pub async fn start_listener(address: SocketAddr) -> Result<(), server::Error> {
    let listener = match TcpListener::bind(&address).await {
        Ok(listener) => listener,
        Err(e) => {
            log::error!("failed to bind to {}: {}", address, e);
            return Err(server::Error::IoError(e));
        }
    };

    log::info!("listening for controllers on: {}", address);

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                tokio::spawn(accept_stream(stream, addr));
            }
            Err(e) => {
                log::error!("failed to accept connection: {}", e);
                // Continue listening on error
            }
        }
    }
}

async fn accept_stream(stream: TcpStream, address: SocketAddr) -> Result<(), server::Error> {
    let ws_stream = tokio_tungstenite::accept_async(stream).await.map_err(|e| {
        log::error!("WebSocket handshake error with {}: {}", address, e);
        server::Error::from(io::Error::other(e))
    })?;

    log::info!("new controller connection: {}", address);

    let (_write, mut read) = ws_stream.split();

    if let Some(msg_result) = read.next().await {
        match msg_result {
            Ok(WsMessage::Text(text)) => {
                serde_json::from_str::<ServerControllerPacket>(&text).map(handle_packet).map_err(
                    |e| {
                        log::error!("failed to decode json Message from {}: {}", address, e);
                        server::Error::from(io::Error::other(e))
                    },
                )?;
            }
            Ok(WsMessage::Binary(bin)) => {
                serde_json::from_slice::<ServerControllerPacket>(&bin).map(handle_packet).map_err(
                    |e| {
                        log::error!("failed to decode binary json Message from {}: {}", address, e);
                        server::Error::from(io::Error::other(e))
                    },
                )?;
            }
            Ok(other) => {
                log::warn!("received non-json packet from {}: {:?}", address, other);
            }
            Err(e) => {
                log::error!("WebSocket error from {}: {}", address, e);
                return Err(server::Error::from(io::Error::other(e)));
            }
        }
    }

    Ok(())
}

fn handle_packet(packet: ServerControllerPacket) {
    eprintln!("handle packet: {packet:?}");
}
