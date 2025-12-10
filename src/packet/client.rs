use crate::dmx::Multiverse;
use crate::engine::BakedPatch;
use crate::packet::PacketPayload;

/// Packets sent from the server to the client.
#[derive(Debug, Clone)]
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

impl PacketPayload for ClientboundPacketPayload {
    fn id(&self) -> u8 {
        match self {
            Self::ResponseBakedPatch(_) => 0,
            Self::ResponseDmxOutput(_) => 1,
            Self::ResponseTriggers => 2,
            Self::ResponseAttributeValues => 3,
            Self::ResponseSetAttributeValues => 4,
        }
    }

    fn from_id_and_data(id: u8, data: &[u8]) -> Result<Self, super::Error> {
        match id {
            0 => {
                let baked_patch =
                    rmp_serde::from_slice(data).map_err(|_| super::Error::InvalidPayload {
                        message: "failed to deserialize BakedPatch".to_string(),
                    })?;

                Ok(Self::ResponseBakedPatch(baked_patch))
            }
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
            Self::ResponseBakedPatch(baked_patch) => {
                rmp_serde::to_vec(baked_patch).map_err(|_| super::Error::InvalidPayload {
                    message: "failed to serialize BakedPatch".to_string(),
                })
            }
            Self::ResponseDmxOutput(multiverse) => {
                rmp_serde::to_vec(multiverse).map_err(|_| super::Error::InvalidPayload {
                    message: "failed to serialize Multiverse".to_string(),
                })
            }
            Self::ResponseTriggers
            | Self::ResponseAttributeValues
            | Self::ResponseSetAttributeValues => Ok(Vec::new()),
        }
    }
}
