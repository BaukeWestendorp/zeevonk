use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::packet::processor::{ClientPacket, ServerPacket};
use crate::server;
use crate::server::state::State;

use tokio::sync::mpsc;

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
            ServerPacket::RegisterClient { id } => {
                state.trigger_router.write().await.register_processor_client(address, id);
                writer.send(ClientPacket::ConfirmRegisterClient).await.map_err(io::Error::other)?;
            }
            ServerPacket::RequestShowData => {
                let show_data = state.show_data.read().await.clone();
                writer
                    .send(ClientPacket::ResponseShowData { show_data })
                    .await
                    .map_err(io::Error::other)?;
            }
            ServerPacket::RequestDmxOutput => {
                let dmx_output = state.output_multiverse.read().await.clone();
                writer
                    .send(ClientPacket::ResponseDmxOutput { dmx_output })
                    .await
                    .map_err(io::Error::other)?;
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
