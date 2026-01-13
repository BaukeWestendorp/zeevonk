use std::io::{self, ErrorKind};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

use crate::server;
use crate::server::state::State;
use crate::trigger::Trigger;

pub async fn start_listener(port: u16, state: Arc<State>) -> Result<(), server::Error> {
    let address = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, port));

    let listener = match TcpListener::bind(&address).await {
        Ok(listener) => listener,
        Err(e) => {
            log::error!("failed to bind to {}: {}", address, e);
            return Err(server::Error::from(e));
        }
    };

    log::info!("listening for processors on: {}", address);

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, addr, state).await {
                        log::error!("client handler for {} failed: {}", addr, e);
                    }
                });
            }
            Err(e) => {
                log::error!("failed to accept connection: {}", e);
            }
        }
    }
}

async fn handle_client(
    stream: TcpStream,
    address: SocketAddr,
    state: Arc<State>,
) -> Result<(), server::Error> {
    let ws_stream = accept_async(stream).await.map_err(|e| {
        log::error!("WebSocket handshake error with {}: {}", address, e);
        server::Error::from(io::Error::new(ErrorKind::Other, e.to_string()))
    })?;

    log::info!("new processor connection: {}", address);

    let (mut write, mut read) = ws_stream.split();

    let (tx, mut rx) = mpsc::channel::<ClientPacket>(16);

    let writer_addr = address.clone();
    let writer_task = tokio::spawn(async move {
        while let Some(packet) = rx.recv().await {
            let msg = match serde_json::to_string(&packet) {
                Ok(json) => Message::Text(json.into()),
                Err(e) => {
                    log::error!(
                        "failed to serialize ClientPacket for {}: {}; sending Close",
                        writer_addr,
                        e
                    );
                    // send a Close and break
                    let _ = write.send(Message::Close(None)).await;
                    break;
                }
            };

            if let Err(e) = write.send(msg).await {
                log::error!("failed to send message to {}: {}", writer_addr, e);
                break;
            }
        }

        // try to close cleanly
        let _ = write.close().await;
    });

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => match serde_json::from_str::<ServerPacket>(&text) {
                Ok(packet) => {
                    handle_packet(packet, address, &tx, Arc::clone(&state)).await?;
                }
                Err(e) => {
                    log::error!("failed to decode json Message (text) from {}: {}", address, e);
                }
            },
            Ok(Message::Binary(bin)) => match serde_json::from_slice::<ServerPacket>(&bin) {
                Ok(packet) => {
                    handle_packet(packet, address, &tx, Arc::clone(&state)).await?;
                }
                Err(e) => {
                    log::error!("failed to decode binary json Message from {}: {}", address, e);
                }
            },
            Ok(Message::Close(_)) => {
                log::info!("received close message from {}: closing connection", address);
                break;
            }
            Ok(_) => {}
            Err(e) => {
                log::error!("WebSocket error from {}: {}", address, e);
                break;
            }
        }
    }

    state.trigger_router.write().await.unregister_controller_client(address);

    // Drop tx so writer task will exit.
    drop(tx);

    // Wait for writer task to finish.
    let _ = writer_task.await;

    Ok(())
}

async fn handle_packet(
    packet: ServerPacket,
    address: SocketAddr,
    writer: &mpsc::Sender<ClientPacket>,
    state: Arc<State>,
) -> Result<(), server::Error> {
    match packet {
        ServerPacket::RegisterClient { name } => {
            state
                .trigger_router
                .write()
                .await
                .register_controller_client(address, name.to_string());

            writer.send(ClientPacket::ConfirmRegisterClient).await.map_err(|e| {
                log::error!(
                    "failed to send ConfirmRegisterClient to writer for {}: {}",
                    address,
                    e
                );
                server::Error::from(io::Error::new(
                    ErrorKind::BrokenPipe,
                    format!("failed to send: {}", e),
                ))
            })?;
        }
        ServerPacket::Trigger(trigger) => {
            state.trigger_router.write().await.handle_trigger(address, trigger);
        }
    }

    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ServerPacket {
    RegisterClient { name: String },
    Trigger(Trigger),
}

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ClientPacket {
    ConfirmRegisterClient,
}
