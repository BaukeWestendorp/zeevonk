use std::collections::HashMap;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::SinkExt;
use futures_util::stream::StreamExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, RwLock, mpsc};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{WebSocketStream, accept_async};

use crate::attr::Attribute;
use crate::ident::Identifier;
use crate::packet::{ClientboundPacket, ServerboundPacket};
use crate::project::Project;
use crate::project::stage::FixtureId;
use crate::server::output::agent::OutputAgent;
use crate::server::router::Router;
use crate::trigger::Trigger;
use crate::value::ClampedValue;

pub struct ClientAgent {
    clients: Mutex<HashMap<Identifier, Arc<ClientHandler>>>,
}

impl ClientAgent {
    pub fn new() -> Self {
        Self { clients: Mutex::new(HashMap::new()) }
    }

    pub async fn register(&self, client_id: Identifier, conn: Arc<ClientHandler>) {
        let mut clients = self.clients.lock().await;
        if clients.contains_key(&client_id) {
            log::warn!("client with id '{}' already exists", client_id);
            return;
        }

        clients.insert(client_id.clone(), conn);
        log::info!("client with id '{}' registered.", client_id);
    }

    pub async fn unregister(&self, client_id: &Identifier) {
        let mut clients = self.clients.lock().await;
        if clients.remove(client_id).is_some() {
            log::info!("client with id '{}' unregistered.", client_id);
            // Successfully removed
        } else {
            log::warn!("client with id '{}' does not exist, cannot unregister.", client_id);
        }
    }

    pub async fn send_trigger(
        &self,
        from_client_id: Identifier,
        to_client_id: Identifier,
        trigger: Trigger,
    ) -> crate::server::Result<()> {
        let clients = self.clients.lock().await;
        if let Some(conn) = clients.get(&to_client_id) {
            conn.send_trigger(from_client_id, trigger).await
        } else {
            Err(crate::server::Error::ClientNotFound(to_client_id.clone()))
        }
    }
}

pub struct ClientListener;

impl ClientListener {
    pub async fn start(
        client_agent: Arc<ClientAgent>,
        output_agent: Arc<OutputAgent>,
        router: Arc<Router>,
        project: Arc<Project>,
        port: u16,
    ) -> crate::server::Result<()> {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr).await?;
        log::info!("listening for clients on {}", addr);

        loop {
            let (stream, peer_addr) = listener.accept().await?;
            let client_agent = client_agent.clone();
            let output_agent = output_agent.clone();
            let router = router.clone();
            let project = project.clone();
            tokio::spawn(async move {
                match accept_async(stream).await {
                    Ok(ws_stream) => {
                        let conn = Arc::new(ClientHandler::new(
                            peer_addr,
                            output_agent.clone(),
                            client_agent.clone(),
                            router.clone(),
                            project.clone(),
                        ));
                        conn.run(ws_stream).await;
                    }
                    Err(e) => log::error!("WebSocket accept error: {}", e),
                }
            });
        }
    }
}

pub struct ClientHandler {
    peer_addr: SocketAddr,
    output_agent: Arc<OutputAgent>,
    client_agent: Arc<ClientAgent>,
    router: Arc<Router>,
    project: Arc<Project>,
    client_id: RwLock<Option<Identifier>>,
    outbound_tx: RwLock<Option<mpsc::UnboundedSender<ClientboundPacket>>>,
}

impl ClientHandler {
    pub fn new(
        peer_addr: SocketAddr,
        output_agent: Arc<OutputAgent>,
        client_agent: Arc<ClientAgent>,
        router: Arc<Router>,
        project: Arc<Project>,
    ) -> Self {
        Self {
            peer_addr,
            output_agent,
            client_agent,
            router,
            project,
            client_id: RwLock::new(None),
            outbound_tx: RwLock::new(None),
        }
    }

    pub async fn run(self: Arc<Self>, mut ws_stream: WebSocketStream<TcpStream>) {
        let (outbound_tx, mut outbound_rx) = mpsc::unbounded_channel();
        *self.outbound_tx.write().await = Some(outbound_tx);

        loop {
            tokio::select! {
                message = ws_stream.next() => {
                    match message {
                        Some(Ok(message)) => if let Err(err) = self.handle_ws_message(message).await {
                            log::error!("error handling packet: {err}");
                        }
                        Some(Err(err)) => {
                            log::error!("WebSocket stream error for client at {}: {}", self.peer_addr, err);
                            break;
                        },
                        None => {
                            log::info!("WebSocket stream closed for client at {}", self.peer_addr);
                            break;
                        }
                    }

                },
                packet = outbound_rx.recv() => {
                    match packet {
                        Some(packet) => {
                            let json = serde_json::to_string(&packet).unwrap().into();
                            let msg = Message::Text(json);
                            if let Err(e) = ws_stream.send(msg).await {
                                log::error!("failed to send packet to client: {}", e);
                                break;
                            }
                        }
                        None => break,
                    }
                }
            }
        }

        if let Some(client_id) = self.client_id.write().await.take() {
            self.client_agent.unregister(&client_id).await;
        }

        log::info!("client connection with {} closed", self.peer_addr);
    }

    async fn handle_ws_message(self: &Arc<Self>, message: Message) -> crate::server::Result<()> {
        match message {
            Message::Text(text) => {
                let Ok(packet) = serde_json::from_str::<ServerboundPacket>(&text) else {
                    return Err(crate::server::Error::PacketDecodingFailed);
                };

                // Register if it has not already.
                if let ServerboundPacket::Register { client_id } = packet {
                    self.client_agent.register(client_id.clone(), Arc::clone(&self)).await;
                    *self.client_id.write().await = Some(client_id);
                    return Ok(());
                }

                let client_id =
                    self.client_id.read().await.clone().expect("should have set the registration");

                match packet {
                    ServerboundPacket::Register { .. } => {
                        unreachable!("we should have already checked registration");
                    }
                    ServerboundPacket::Unregister => {
                        self.client_agent.unregister(&client_id).await;
                    }

                    ServerboundPacket::UpdateAttributes { values, include_children } => {
                        self.output_agent.update_values(values.clone());

                        if include_children {
                            fn set_values_recursively(
                                project: &Project,
                                output_agent: &OutputAgent,
                                fixture_id: &FixtureId,
                                attribute: Attribute,
                                value: ClampedValue,
                            ) {
                                let Some(fixture) = project.stage().fixtures().get(fixture_id)
                                else {
                                    return;
                                };

                                for sub_id in fixture.sub_ids() {
                                    output_agent.update_value(*sub_id, attribute, value);

                                    set_values_recursively(
                                        project,
                                        output_agent,
                                        &sub_id,
                                        attribute,
                                        value,
                                    );
                                }
                            }

                            for (fixture_id, attribute, value) in values.values() {
                                set_values_recursively(
                                    &self.project,
                                    &self.output_agent,
                                    fixture_id,
                                    *attribute,
                                    *value,
                                );
                            }
                        }
                    }

                    ServerboundPacket::RequestProjectData => {
                        let project = (*self.project).clone();
                        self.send_packet(ClientboundPacket::ProjectData { project }).await?;
                    }

                    ServerboundPacket::Trigger { trigger } => {
                        self.router.handle_trigger(&client_id, trigger).await;
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    async fn send_packet(&self, packet: ClientboundPacket) -> crate::server::Result<()> {
        let outbound_tx = self.outbound_tx.read().await;
        let outbound_tx = outbound_tx.as_ref().ok_or_else(|| {
            crate::server::Error::Io(io::Error::other(
                "tried to send packet to client before the handler was started",
            ))
        })?;

        outbound_tx.send(packet).map_err(|_| {
            crate::server::Error::Io(io::Error::other(
                "tried to send packet to client after the handler was closed",
            ))
        })?;

        Ok(())
    }

    pub async fn send_trigger(
        &self,
        from_client_id: Identifier,
        trigger: Trigger,
    ) -> crate::server::Result<()> {
        self.send_packet(ClientboundPacket::Trigger { from_client_id, trigger }).await?;

        Ok(())
    }
}
