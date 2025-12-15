use crate::dmx::Multiverse;
use crate::packet::PacketPayload;
use crate::state::State;

/// Packets sent from the server to the client.
#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ClientPacketPayload {
    ResponseState(State),
    ResponseDmxOutput(Multiverse),
    ResponseSetAttributeValues,
}

impl PacketPayload for ClientPacketPayload {}
