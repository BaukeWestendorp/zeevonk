use crate::showfile::patch::Patch;
use crate::showfile::protocols::Protocols;

pub mod parser;
pub mod patch;
pub mod protocols;

#[derive(Debug, Clone, PartialEq)]
pub struct Showfile {
    patch: Patch,
    protocols: Protocols,
}

impl Showfile {
    pub fn patch(&self) -> &Patch {
        &self.patch
    }

    pub fn protocols(&self) -> &Protocols {
        &self.protocols
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Label(String);

impl Label {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}
