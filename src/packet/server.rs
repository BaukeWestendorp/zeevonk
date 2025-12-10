use crate::gdcs::ClampedValue;
use crate::gdcs::attr::Attribute;
use crate::gdcs::fixture::FixturePath;
use crate::packet::PacketPayload;

/// Packets sent from the client to the server.
#[derive(Debug, Clone, PartialEq)]
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
    RequestSetAttributeValues {
        /// values.0: The path of the (sub)fixture whose attribute is to be set.
        ///
        /// values.1: The attribute to set.
        ///
        /// values.2: The new value for the attribute.
        values: Vec<(FixturePath, Attribute, ClampedValue)>,
    },
}

impl PacketPayload for ServerboundPacketPayload {
    fn id(&self) -> u8 {
        match self {
            Self::RequestLayout => 0,
            Self::RequestDmxOutput => 1,
            Self::RequestTriggers => 2,
            Self::RequestAttributeValues => 3,
            Self::RequestSetAttributeValues { .. } => 4,
        }
    }

    fn from_id_and_data(id: u8, data: &[u8]) -> Result<Self, super::Error> {
        match id {
            0 => Ok(Self::RequestLayout),
            1 => Ok(Self::RequestDmxOutput),
            2 => Ok(Self::RequestTriggers),
            3 => Ok(Self::RequestAttributeValues),
            4 => {
                let values =
                    rmp_serde::from_slice(data).map_err(|_| super::Error::InvalidPayload {
                        message: "failed to deserialize values".to_string(),
                    })?;

                Ok(Self::RequestSetAttributeValues { values })
            }
            _ => Err(super::Error::InvalidPacketId(id)),
        }
    }

    fn to_data_bytes(&self) -> Result<Vec<u8>, super::Error> {
        match self {
            ServerboundPacketPayload::RequestLayout
            | ServerboundPacketPayload::RequestDmxOutput
            | ServerboundPacketPayload::RequestTriggers
            | ServerboundPacketPayload::RequestAttributeValues => Ok(Vec::new()),
            ServerboundPacketPayload::RequestSetAttributeValues { values } => {
                rmp_serde::to_vec(values).map_err(|_| super::Error::InvalidPayload {
                    message: "failed to serialize values".to_string(),
                })
            }
        }
    }
}
