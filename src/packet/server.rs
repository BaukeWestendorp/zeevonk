use crate::packet::PacketPayload;

/// Packets sent from the client to the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerboundPacketPayload {
    /// Request the current layout from the server.
    RequestLayout,
    /// Request the current DMX output data of all universes from the server.
    RequestDmxOutput,
    /// Request the current triggers from the server.
    RequestTriggers,
    /// Request the current attribute values from the server.
    RequestAttributeValues,
    /// Request to set new attribute values on the server.
    RequestSetAttributeValues,
}

impl PacketPayload for ServerboundPacketPayload {
    fn id(&self) -> u8 {
        match self {
            Self::RequestLayout => 0,
            Self::RequestDmxOutput => 1,
            Self::RequestTriggers => 2,
            Self::RequestAttributeValues => 3,
            Self::RequestSetAttributeValues => 4,
        }
    }

    fn from_id_and_data(id: u8, _data: &[u8]) -> Result<Self, super::Error> {
        match id {
            0 => Ok(Self::RequestLayout),
            1 => Ok(Self::RequestDmxOutput),
            2 => Ok(Self::RequestTriggers),
            3 => Ok(Self::RequestAttributeValues),
            4 => Ok(Self::RequestSetAttributeValues),
            _ => Err(super::Error::InvalidPacketId(id)),
        }
    }

    fn to_data_bytes(&self) -> Result<Vec<u8>, super::Error> {
        Ok(Vec::new())
    }
}
