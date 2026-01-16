//! Patch management for collections of fixtures.

use std::collections::BTreeMap;

use crate::fixture::{Fixture, FixtureId};

/// A patch containing a set of [`Fixture`]s.
#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Patch {
    pub(crate) fixtures: BTreeMap<FixtureId, Fixture>,
}

impl Patch {
    /// Returns the map of fixtures contained in this patch.
    pub fn fixtures(&self) -> &BTreeMap<FixtureId, Fixture> {
        &self.fixtures
    }
}
