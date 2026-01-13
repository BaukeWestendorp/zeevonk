use crate::Identifier;

#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trigger {
    id: Identifier,
    value: TriggerValue,
}

impl Trigger {
    pub fn new(id: Identifier, value: TriggerValue) -> Self {
        Self { id: id.into(), value }
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }

    pub fn value(&self) -> &TriggerValue {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub enum TriggerValue {
    #[serde(rename = "empty")]
    Empty,
    #[serde(rename = "integer")]
    Integer(i64),
    #[serde(rename = "float")]
    Float(f64),
    #[serde(rename = "boolean")]
    Boolean(bool),
}
