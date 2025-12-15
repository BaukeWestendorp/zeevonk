use crate::packet::{AttributeValues, PacketPayload};

/// Packets sent from the client to the server.
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ServerPacketPayload {
    RequestShowData,
    RequestDmxOutput,
    RequestSetAttributeValues(AttributeValues),
}

impl PacketPayload for ServerPacketPayload {}
