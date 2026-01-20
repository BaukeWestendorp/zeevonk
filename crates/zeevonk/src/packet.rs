//! Packet definitions for both clients and the server.

/// General packet that can be serialized and deserialized using [`serde`].
pub trait Packet: serde::Serialize + for<'de> serde::Deserialize<'de> {}

/// Packets used by the controller client.
pub mod controller {
    use crate::ident::Identifier;
    use crate::packet::Packet;
    use crate::trigger::Trigger;

    /// Packets sent from the controller client to the server.
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
    pub enum ClientboundPacket {
        /// The server successfully registered this client.
        RegisterSuccess,
    }

    impl Packet for ClientboundPacket {}
}

/// Packets used by the processor client.
pub mod processor {
    use crate::ident::Identifier;
    use crate::packet::Packet;
    use crate::trigger::Trigger;
    use crate::value::AttributeValues;

    /// Packets sent from the processor client to the server.
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
    pub enum ClientboundPacket {
        /// A trigger that has been sent from a controller.
        Trigger {
            /// The client that sent the trigger.
            from_client_id: Identifier,
            /// The trigger.
            trigger: Trigger,
        },
    }

    impl Packet for ClientboundPacket {}
}
