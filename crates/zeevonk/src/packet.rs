//! Packet definitions for both clients and the server.

/// General packet that can be serialized and deserialized using [`serde`].
pub trait Packet: serde::Serialize + for<'de> serde::Deserialize<'de> {}

/// Packets used by the controller client.
pub mod controller {
    use crate::packet::Packet;
    use crate::trigger::Trigger;

    /// Packets sent from the controller client to the server.
    #[derive(Debug, Clone)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum ServerboundPacket {
        /// Request that the server process a trigger.
        Trigger {
            /// The trigger to be processed.
            trigger: Trigger,
        },
    }

    impl Packet for ServerboundPacket {}

    /// Packets sent from the server to a controller client.
    #[derive(Debug, Clone)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum ClientboundPacket {}

    impl Packet for ClientboundPacket {}
}

/// Packets used by the processor client.
pub mod processor {
    use crate::ident::Identifier;
    use crate::packet::Packet;
    use crate::value::AttributeValues;

    /// Packets sent from the processor client to the server.
    #[derive(Debug, Clone)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum ServerboundPacket {
        /// Register this client with the processor.
        RegisterClient {
            /// The identifier for the client to register.
            id: Identifier,
        },

        /// Update one or more attribute values on the processor.
        UpdateAttributes {
            /// The attribute values to update.
            values: AttributeValues,
            /// Whether updates propagate attribute values to child fixtures.
            #[serde(default)]
            include_children: bool,
        },
    }

    impl Packet for ServerboundPacket {}

    /// Packets sent from the server to a processor client.
    #[derive(Debug, Clone)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum ClientboundPacket {}

    impl Packet for ClientboundPacket {}
}
