use crate::dmx::Multiverse;
use crate::packet::PacketPayload;

/// Packets sent from the server to the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientboundPacketPayload {
    /// Response containing the layout information.
    ResponseLayout,

    /// Response containing DMX output data for all universes.
    ResponseDmxOutput(Multiverse),

    /// Response containing trigger information.
    ResponseTriggers,

    /// Response containing attribute values.
    ResponseAttributeValues,

    /// Response confirming that attribute values have been set.
    ResponseSetAttributeValues,
}

impl PacketPayload for ClientboundPacketPayload {
    fn id(&self) -> u8 {
        match self {
            Self::ResponseLayout => 0,
            Self::ResponseDmxOutput(_) => 1,
            Self::ResponseTriggers => 2,
            Self::ResponseAttributeValues => 3,
            Self::ResponseSetAttributeValues => 4,
        }
    }

    fn from_id_and_data(id: u8, data: &[u8]) -> Result<Self, super::Error> {
        match id {
            0 => Ok(Self::ResponseLayout),
            1 => {
                let multiverse =
                    rmp_serde::from_slice(data).map_err(|_| super::Error::InvalidPayload {
                        message: "failed to deserialize Multiverse".to_string(),
                    })?;

                Ok(Self::ResponseDmxOutput(multiverse))
            }
            2 => Ok(Self::ResponseTriggers),
            3 => Ok(Self::ResponseAttributeValues),
            4 => Ok(Self::ResponseSetAttributeValues),
            _ => Err(super::Error::InvalidPacketId(id)),
        }
    }

    fn to_data_bytes(&self) -> Result<Vec<u8>, super::Error> {
        match self {
            Self::ResponseLayout
            | Self::ResponseTriggers
            | Self::ResponseAttributeValues
            | Self::ResponseSetAttributeValues => Ok(Vec::new()),
            Self::ResponseDmxOutput(multiverse) => {
                rmp_serde::to_vec(multiverse).map_err(|_| super::Error::InvalidPayload {
                    message: "failed to serialize Multiverse".to_string(),
                })
            }
        }
    }
}
