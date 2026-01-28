//! Represents a trigger sent from a client to be processed and routed by the server.
//!
//! The server will route any incoming triggers to the correct client according to the project configuration.

use crate::ident::Identifier;

/// A trigger can be sent from a client, and will be processed by the server.
/// The server will route any incoming triggers to the correct client according to the project configuration.
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Trigger {
    id: Identifier,
    value: TriggerValue,
}

impl Trigger {
    /// Create a new [`Trigger`].
    pub fn new(id: Identifier, value: TriggerValue) -> Self {
        Self { id, value }
    }

    /// Return the [`Identifier`].
    pub fn id(&self) -> &Identifier {
        &self.id
    }

    /// Return the [`TriggerValue`].
    pub fn value(&self) -> &TriggerValue {
        &self.value
    }
}

/// The possible values a [`Trigger`] can carry.
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum TriggerValue {
    /// No value is associated with the trigger.
    Empty,
    /// A signed 64-bit integer value.
    Integer(i64),
    /// A 64-bit floating point value.
    Float(f64),
    /// A boolean value.
    Boolean(bool),
}
