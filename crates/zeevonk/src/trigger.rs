use crate::Identifier;

#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
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
    Empty,
    Integer(i64),
    Float(f64),
    Boolean(bool),
}
