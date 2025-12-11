use crate::core::dmx::Multiverse;
use crate::core::packet::PacketPayload;
use crate::server::BakedPatch;

/// Packets sent from the server to the client.
#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum ClientboundPacketPayload {
    /// Response containing the baked patch information.
    ResponseBakedPatch(BakedPatch),
    /// Response containing DMX output data for all universes.
    ResponseDmxOutput(Multiverse),
    /// Response confirming that attribute values have been set.
    ResponseSetAttributeValues,
}

impl PacketPayload for ClientboundPacketPayload {}
