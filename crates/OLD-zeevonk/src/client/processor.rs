use std::io;
use std::net::SocketAddr;
use tokio::sync::{Mutex, mpsc};

use crate::Identifier;
use crate::client::{self, ClientHandle};
use crate::packet::processor::{ClientPacket, ServerPacket};
use crate::value::AttributeValues;

pub struct ProcessorClient {
    id: Identifier,
    server_address: SocketAddr,
    handle: Mutex<Option<ClientHandle<ServerPacket>>>,
}

impl ProcessorClient {
    pub fn new(id: Identifier, server_address: SocketAddr) -> Self {
        Self { id, server_address, handle: Mutex::new(None) }
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }

    pub async fn connect(&self) -> Result<(), client::Error> {
        if self.handle.lock().await.is_some() {
            log::warn!("processor client is already connected");
            return Ok(());
        }

        let handle = super::start_ws_client::<Self>(self.id.clone(), self.server_address).await?;
        *self.handle.lock().await = Some(handle);

        log::info!("processor client connected to server at {}", self.server_address);

        Ok(())
    }

    pub async fn disconnect(&self) {
        self.handle.lock().await.take();
        log::info!("processor client disconnecting from server at {}", self.server_address);
    }

    pub async fn set_attribute_values(&self, values: AttributeValues) -> Result<(), client::Error> {
        let guard = self.handle.lock().await;
        let handle =
            guard.as_ref().ok_or_else(|| io::Error::other("processor client not connected"))?;

        handle
            .send(ServerPacket::SetAttributeValues { values, include_children: false })
            .await
            .map_err(|e| io::Error::other(e))?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl super::ClientPacketHandler for ProcessorClient {
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
        log::info!("processor client disconnected");
    }

    async fn handle_packet(
        packet: ClientPacket,
        _writer: &mpsc::Sender<ServerPacket>,
    ) -> Result<(), client::Error> {
        match packet {
            ClientPacket::ConfirmRegisterClient => {
                log::info!("processor registration confirmed");
            }
            ClientPacket::ResponseShowData { .. } => todo!(),
            ClientPacket::ResponseDmxOutput { .. } => todo!(),
        }
        Ok(())
    }
}
