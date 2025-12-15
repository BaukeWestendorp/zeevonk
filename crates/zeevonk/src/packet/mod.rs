use std::collections::HashMap;

pub use client::*;
#[cfg(feature = "tokio")]
pub use codec::*;
pub use error::*;
pub use server::*;

use crate::attr::Attribute;
use crate::state::fixture::FixturePath;
use crate::value::ClampedValue;

mod client;
#[cfg(feature = "tokio")]
mod codec;
mod error;
mod server;

/// Trait for types that can be used as packet payloads.
pub trait PacketPayload: serde::Serialize + for<'de> serde::Deserialize<'de> {}

/// A packet containing a payload.
#[derive(Debug)]
pub struct Packet<P: PacketPayload> {
    pub payload: P,
}

impl<P: PacketPayload> Packet<P> {
    pub fn new(payload: P) -> Self {
        Self { payload }
    }

    pub fn decode_payload_bytes(payload_bytes: &[u8]) -> Result<Self, Error> {
        let payload = rmp_serde::from_slice(payload_bytes)
            .map_err(|err| Error::InvalidPayload { message: err.to_string() })?;
        let packet = Packet { payload };
        Ok(packet)
    }

    pub fn encode_payload_bytes(&self) -> Result<Vec<u8>, Error> {
        rmp_serde::to_vec(&self.payload)
            .map_err(|err| Error::InvalidPayload { message: err.to_string() })
    }
}

#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct AttributeValues {
    values: HashMap<(FixturePath, Attribute), ClampedValue>,
}

impl AttributeValues {
    pub fn new() -> Self {
        Self { values: HashMap::new() }
    }

    pub fn set(
        &mut self,
        fixture_path: FixturePath,
        attribute: Attribute,
        value: impl Into<ClampedValue>,
    ) {
        self.values.insert((fixture_path, attribute), value.into());
    }

    pub fn values(&self) -> impl Iterator<Item = (&(FixturePath, Attribute), &ClampedValue)> {
        self.values.iter()
    }

    pub fn get(&self, path: FixturePath, attribute: Attribute) -> Option<ClampedValue> {
        self.values.get(&(path, attribute)).copied()
    }
}
