use crate::core::packet::PacketPayload;
use crate::server::AttributeValues;

/// Packets sent from the client to the server.
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum ServerboundPacketPayload {
    /// Request the current baked patch from the server.
    RequestBakedPatch,
    /// Request the current DMX output data of all universes from the server.
    RequestDmxOutput,
    /// Request to set new attribute values on the server.
    RequestSetAttributeValues {
        /// Attribute values to set.
        values: AttributeValues,
    },
}

impl PacketPayload for ServerboundPacketPayload {}
