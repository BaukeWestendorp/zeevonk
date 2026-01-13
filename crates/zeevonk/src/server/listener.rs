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
    let ws_stream = accept_async(stream)
        .await
        .map_err(|e| server::Error::from(io::Error::new(ErrorKind::Other, e.to_string())))?;

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

pub mod controller {
    use std::io;
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tokio::sync::mpsc;

    use crate::server;
    use crate::server::state::State;
    use crate::trigger::Trigger;

    pub struct ControllerHandler;

    #[async_trait::async_trait]
    impl super::PacketHandler for ControllerHandler {
        type ServerPacket = ServerPacket;
        type ClientPacket = ClientPacket;

        async fn on_disconnect(address: SocketAddr, state: Arc<State>) {
            state.trigger_router.write().await.unregister_controller_client(address);
        }

        async fn handle_packet(
            packet: ServerPacket,
            address: SocketAddr,
            writer: &mpsc::Sender<ClientPacket>,
            state: Arc<State>,
        ) -> Result<(), server::Error> {
            match packet {
                ServerPacket::RegisterClient { name } => {
                    state.trigger_router.write().await.register_controller_client(address, name);

                    writer
                        .send(ClientPacket::ConfirmRegisterClient)
                        .await
                        .map_err(|e| io::Error::other(e))?;
                }
                ServerPacket::Trigger { trigger } => {
                    state.trigger_router.write().await.handle_trigger(address, trigger);
                }
            }
            Ok(())
        }
    }

    #[derive(serde::Deserialize)]
    pub enum ServerPacket {
        RegisterClient { name: String },
        Trigger { trigger: Trigger },
    }

    #[derive(serde::Serialize)]
    pub enum ClientPacket {
        ConfirmRegisterClient,
    }
}

pub mod processor {
    use std::io;
    use std::net::SocketAddr;
    use std::sync::Arc;
    use theymx::Multiverse;
    use tokio::sync::mpsc;

    use crate::attr::Attribute;
    use crate::server;
    use crate::server::state::State;
    use crate::show::ShowData;
    use crate::show::fixture::FixturePath;
    use crate::value::ClampedValue;

    pub struct ProcessorHandler;

    #[async_trait::async_trait]
    impl super::PacketHandler for ProcessorHandler {
        type ServerPacket = ServerPacket;
        type ClientPacket = ClientPacket;

        async fn on_disconnect(address: SocketAddr, state: Arc<State>) {
            state.trigger_router.write().await.unregister_processor_client(address);
        }

        async fn handle_packet(
            packet: ServerPacket,
            address: SocketAddr,
            writer: &mpsc::Sender<ClientPacket>,
            state: Arc<State>,
        ) -> Result<(), server::Error> {
            match packet {
                ServerPacket::RegisterClient { name } => {
                    state.trigger_router.write().await.register_processor_client(address, name);
                    writer
                        .send(ClientPacket::ConfirmRegisterClient)
                        .await
                        .map_err(|e| io::Error::other(e))?;
                }
                ServerPacket::RequestShowData => {
                    let show_data = state.show_data.read().await.clone();
                    writer
                        .send(ClientPacket::ResponseShowData { show_data })
                        .await
                        .map_err(|e| io::Error::other(e))?;
                }
                ServerPacket::RequestDmxOutput => {
                    let dmx_output = state.output_multiverse.read().await.clone();
                    writer
                        .send(ClientPacket::ResponseDmxOutput { dmx_output })
                        .await
                        .map_err(|e| io::Error::other(e))?;
                }
                ServerPacket::SetAttributeValues {
                    fixture_path,
                    attribute,
                    value,
                    include_children,
                } => {
                    if include_children {
                        todo!();
                    }

                    state.set_attribute_value(fixture_path, attribute, value).await;
                }
            }
            Ok(())
        }
    }

    #[derive(serde::Deserialize)]
    pub enum ServerPacket {
        RegisterClient {
            name: String,
        },
        RequestShowData,
        RequestDmxOutput,
        SetAttributeValues {
            fixture_path: FixturePath,
            attribute: Attribute,
            value: ClampedValue,
            include_children: bool,
        },
    }

    #[derive(serde::Serialize)]
    pub enum ClientPacket {
        ConfirmRegisterClient,
        ResponseShowData { show_data: ShowData },
        ResponseDmxOutput { dmx_output: Multiverse },
    }
}
