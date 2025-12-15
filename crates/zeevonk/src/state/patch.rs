use std::collections::BTreeMap;

use crate::dmx::Multiverse;
use crate::state::fixture::{Fixture, FixturePath};

#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Patch {
    pub(crate) fixtures: BTreeMap<FixturePath, Fixture>,
    pub(crate) default_multiverse: Multiverse,
}

impl Patch {
    pub fn fixtures(&self) -> &BTreeMap<FixturePath, Fixture> {
        &self.fixtures
    }

    pub fn default_multiverse(&self) -> &Multiverse {
        &self.default_multiverse
    }
}
