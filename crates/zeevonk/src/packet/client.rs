use crate::dmx::Multiverse;
use crate::packet::PacketPayload;
use crate::show::ShowData;

/// Packets sent from the server to the client.
#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ClientPacketPayload {
    ResponseShowData(ShowData),
    ResponseDmxOutput(Multiverse),
    ResponseSetAttributeValues,
}

impl PacketPayload for ClientPacketPayload {}
