pub mod controller {
    use crate::trigger::Trigger;

    #[derive(serde::Deserialize)]
    pub enum ServerPacket {
        RegisterClient { name: String },
        Trigger { trigger: Trigger },
    }

    #[derive(serde::Serialize)]
    pub enum ClientPacket {
        ConfirmRegisterClient,
    }
}

pub mod processor {
    use crate::attr::Attribute;
    use crate::show::ShowData;
    use crate::show::fixture::FixturePath;
    use crate::value::ClampedValue;
    use theymx::Multiverse;

    #[derive(serde::Deserialize)]
    pub enum ServerPacket {
        RegisterClient {
            name: String,
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

    #[derive(serde::Serialize)]
    pub enum ClientPacket {
        ConfirmRegisterClient,
        ResponseShowData { show_data: ShowData },
        ResponseDmxOutput { dmx_output: Multiverse },
    }
}
