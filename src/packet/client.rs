use crate::dmx::Multiverse;
use crate::engine::BakedPatch;
use crate::packet::PacketPayload;

/// Packets sent from the server to the client.
#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum ClientboundPacketPayload {
    /// Response containing the baked patch information.
    ResponseBakedPatch(BakedPatch),
    /// Response containing DMX output data for all universes.
    ResponseDmxOutput(Multiverse),
    /// Response containing trigger information.
    ResponseTriggers,
    /// Response containing attribute values.
    ResponseAttributeValues,
    /// Response confirming that attribute values have been set.
    ResponseSetAttributeValues,
}

impl PacketPayload for ClientboundPacketPayload {}
