use std::io;
use std::net::SocketAddr;
use tokio::sync::{Mutex, mpsc};

use crate::Identifier;
use crate::client::{self, ClientHandle};
use crate::packet::controller::{ClientPacket, ServerPacket};
use crate::trigger::Trigger;

pub struct ControllerClient {
    id: Identifier,
    server_address: SocketAddr,
    handle: Mutex<Option<ClientHandle<ServerPacket>>>,
}

impl ControllerClient {
    pub fn new(id: Identifier, server_address: SocketAddr) -> Self {
        Self { id, server_address, handle: Mutex::new(None) }
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }

    pub async fn connect(&self) -> Result<(), client::Error> {
        if self.handle.lock().await.is_some() {
            log::warn!("controller client is already connected");
            return Ok(());
        }

        let handle = super::start_ws_client::<Self>(self.id.clone(), self.server_address).await?;
        *self.handle.lock().await = Some(handle);

        log::info!("controller client connected to server at {}", self.server_address);

        Ok(())
    }

    pub async fn disconnect(&self) {
        self.handle.lock().await.take();
        log::info!("controller client disconnecting from server at {}", self.server_address);
    }

    pub async fn send_trigger(&self, trigger: Trigger) -> Result<(), client::Error> {
        let guard = self.handle.lock().await;
        let handle =
            guard.as_ref().ok_or_else(|| io::Error::other("controller client not connected"))?;

        handle.send(ServerPacket::Trigger { trigger }).await.map_err(|e| io::Error::other(e))?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl super::ClientPacketHandler for ControllerClient {
    type ServerPacket = ServerPacket;
    type ClientPacket = ClientPacket;

    async fn on_connect(
        id: Identifier,
        writer: &mpsc::Sender<Self::ServerPacket>,
    ) -> Result<(), client::Error> {
        writer.send(ServerPacket::RegisterClient { id }).await.map_err(|e| io::Error::other(e))?;
        Ok(())
    }

    async fn on_disconnect() {
        log::info!("controller client disconnected");
    }

    async fn handle_packet(
        packet: ClientPacket,
        _writer: &mpsc::Sender<ServerPacket>,
    ) -> Result<(), client::Error> {
        match packet {
            ClientPacket::ConfirmRegisterClient => {
                log::info!("controller registration confirmed");
            }
        }
        Ok(())
    }
}
