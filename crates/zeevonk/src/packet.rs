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
    use crate::show::ShowData;
    use crate::value::AttributeValues;
    use theymx::Multiverse;

    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum ServerPacket {
        RegisterClient { id: Identifier },
        RequestShowData,
        RequestDmxOutput,
        SetAttributeValues { values: AttributeValues, include_children: bool },
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    pub enum ClientPacket {
        ConfirmRegisterClient,
        ResponseShowData { show_data: ShowData },
        ResponseDmxOutput { dmx_output: Multiverse },
    }
}
