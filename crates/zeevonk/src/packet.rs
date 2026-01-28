//! Packet definitions for both clients and the server.

use crate::ident::Identifier;
use crate::project::Project;
use crate::trigger::Trigger;
use crate::value::AttributeValues;

/// General packet that can be serialized and deserialized using [`serde`].
pub trait Packet: serde::Serialize + for<'de> serde::Deserialize<'de> {}

/// Packets sent from a client to the server.
#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum ServerboundPacket {
    /// Register this client at the server to prepare for routing.
    Register {
        /// The identifier for the client to register.
        client_id: Identifier,
    },
    /// Unregister this client at the server.
    Unregister,

    /// Request the server to provide project data.
    RequestProjectData,

    /// Update one or more attribute values.
    UpdateAttributes {
        /// The attribute values to update.
        values: AttributeValues,
        /// Whether updates propagate attribute values to child fixtures.
        #[serde(default)]
        include_children: bool,
    },

    /// Request that the server process a trigger.
    Trigger {
        /// The trigger to be processed.
        trigger: Trigger,
    },
}

impl Packet for ServerboundPacket {}

/// Packets sent from the server to a client.
#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum ClientboundPacket {
    /// The server successfully registered this client.
    RegisterSuccess,

    /// A trigger that has been sent from a client.
    Trigger {
        /// The id of the client that sent the trigger.
        from_client_id: Identifier,
        /// The trigger.
        trigger: Trigger,
    },

    /// Project data sent from the server.
    ProjectData {
        /// The project data.
        project: Project,
    },
}

impl Packet for ClientboundPacket {}
