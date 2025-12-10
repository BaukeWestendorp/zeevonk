use crate::gdcs::{Attribute, ClampedValue, FixturePath};
use crate::packet::PacketPayload;

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
        /// values.0: The path of the (sub)fixture whose attribute is to be set.
        ///
        /// values.1: The attribute to set.
        ///
        /// values.2: The new value for the attribute.
        values: Vec<(FixturePath, Attribute, ClampedValue)>,
    },
}

impl PacketPayload for ServerboundPacketPayload {}
