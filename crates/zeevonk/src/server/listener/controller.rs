use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::packet::controller::{ClientPacket, ServerPacket};
use crate::server;
use crate::server::state::State;

use tokio::sync::mpsc;

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
            ServerPacket::RegisterClient { id } => {
                state.trigger_router.write().await.register_controller_client(address, id);

                writer.send(ClientPacket::ConfirmRegisterClient).await.map_err(io::Error::other)?;
            }
            ServerPacket::Trigger { trigger } => {
                state.trigger_router.write().await.handle_trigger(address, trigger);
            }
        }
        Ok(())
    }
}
