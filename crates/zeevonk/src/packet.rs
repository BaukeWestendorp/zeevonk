pub mod controller {
    use crate::Identifier;
    use crate::trigger::Trigger;

    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum ServerPacket {
        RegisterClient { id: Identifier },
        Trigger { trigger: Trigger },
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum ClientPacket {
        ConfirmRegisterClient,
    }
}

pub mod processor {
    use crate::Identifier;
    use crate::attr::Attribute;
    use crate::show::ShowData;
    use crate::show::fixture::FixturePath;
    use crate::value::ClampedValue;
    use theymx::Multiverse;

    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum ServerPacket {
        RegisterClient {
            id: Identifier,
        },
        RequestShowData,
        RequestDmxOutput,
        SetAttributeValues {
            fixture_path: FixturePath,
            attribute: Attribute,
            value: ClampedValue,
            include_children: bool,
        },
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum ClientPacket {
        ConfirmRegisterClient,
        ResponseShowData { show_data: ShowData },
        ResponseDmxOutput { dmx_output: Multiverse },
    }
}
