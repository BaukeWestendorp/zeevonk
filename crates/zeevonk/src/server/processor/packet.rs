use crate::trigger::Trigger;

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ServerProcessorPacket {
    RegisterClient { name: String },
    Trigger(Trigger),
}
