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

use crate::ident::Identifier;
use crate::packet::processor::{ClientboundPacket, ServerboundPacket};
use crate::server::output::agent::OutputAgent;
use crate::trigger::Trigger;

pub struct ProcessorManager {
    processors: Mutex<HashMap<Identifier, Arc<ProcessorConnection>>>,
}

impl ProcessorManager {
    pub fn new() -> Self {
        Self { processors: Mutex::new(HashMap::new()) }
    }

    pub async fn register(&self, client_id: Identifier, conn: Arc<ProcessorConnection>) {
        let mut processors = self.processors.lock().await;
        if processors.contains_key(&client_id) {
            log::warn!("processor with id '{}' already exists", client_id);
            return;
        }

        processors.insert(client_id.clone(), conn);
        log::info!("processor with id '{}' registered.", client_id);
    }

    pub async fn unregister(&self, client_id: &Identifier) {
        let mut processors = self.processors.lock().await;
        if processors.remove(client_id).is_some() {
            log::info!("processor with id '{}' unregistered.", client_id);
            // Successfully removed
        } else {
            log::warn!("processor with id '{}' does not exist, cannot unregister.", client_id);
        }
    }

    pub async fn send_trigger(
        &self,
        from_client_id: Identifier,
        to_client_id: Identifier,
        trigger: Trigger,
    ) -> crate::server::Result<()> {
        let processors = self.processors.lock().await;
        if let Some(conn) = processors.get(&to_client_id) {
            conn.send_trigger(from_client_id, trigger).await
        } else {
            Err(crate::server::Error::ClientNotFound(to_client_id.clone()))
        }
    }
}

pub struct ProcessorListener;

impl ProcessorListener {
    pub async fn start(
        agent: Arc<ProcessorManager>,
        output_agent: Arc<OutputAgent>,
        port: u16,
    ) -> crate::server::Result<()> {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr).await?;
        log::info!("listening for processors on {}", addr);

        loop {
            let (stream, peer_addr) = listener.accept().await?;
            let agent = agent.clone();
            let output_agent = output_agent.clone();
            tokio::spawn(async move {
                match accept_async(stream).await {
                    Ok(ws_stream) => {
                        let conn = Arc::new(ProcessorConnection::new(
                            peer_addr,
                            output_agent.clone(),
                            agent.clone(),
                        ));
                        conn.run(ws_stream).await;
                    }
                    Err(e) => log::error!("WebSocket accept error: {}", e),
                }
            });
        }
    }
}

pub struct ProcessorConnection {
    peer_addr: SocketAddr,
    output_agent: Arc<OutputAgent>,
    agent: Arc<ProcessorManager>,
    client_id: RwLock<Option<Identifier>>,
    outbound_tx: RwLock<Option<mpsc::UnboundedSender<ClientboundPacket>>>,
}

impl ProcessorConnection {
    pub fn new(
        peer_addr: SocketAddr,
        output_agent: Arc<OutputAgent>,
        agent: Arc<ProcessorManager>,
    ) -> Self {
        Self {
            peer_addr,
            output_agent,
            agent,
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
                            log::error!("WebSocket stream error for processor at {}: {}", self.peer_addr, err);
                            break;
                        },
                        None => {
                            log::info!("WebSocket stream closed for processor at {}", self.peer_addr);
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
                                log::error!("failed to send packet to processor: {}", e);
                                break;
                            }
                        }
                        None => break,
                    }
                }
            }
        }

        if let Some(client_id) = self.client_id.write().await.take() {
            self.agent.unregister(&client_id).await;
        }

        log::info!("processor connection with {} closed", self.peer_addr);
    }

    async fn handle_ws_message(self: &Arc<Self>, message: Message) -> crate::server::Result<()> {
        match message {
            Message::Text(text) => {
                let Ok(packet) = serde_json::from_str::<ServerboundPacket>(&text) else {
                    return Err(crate::server::Error::PacketDecodingFailed);
                };

                // Register if it has not already.
                if let ServerboundPacket::Register { client_id } = packet {
                    self.agent.register(client_id.clone(), Arc::clone(&self)).await;
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
                        self.agent.unregister(&client_id).await;
                    }
                    ServerboundPacket::UpdateAttributes { values, include_children } => {
                        if include_children {
                            todo!();
                        }

                        self.output_agent.update_values(values);
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
