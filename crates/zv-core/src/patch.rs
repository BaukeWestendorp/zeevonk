//! Patch management for collections of fixtures.

use std::collections::HashMap;

use crate::fixture::{Fixture, FixtureId};

/// A patch containing a set of [`Fixture`]s.
#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Patch {
    fixtures: HashMap<FixtureId, Fixture>,
}

impl Patch {
    pub fn new(fixtures: HashMap<FixtureId, Fixture>) -> Self {
        Self { fixtures }
    }

    /// Returns the map of fixtures contained in this patch.
    pub fn fixtures(&self) -> &HashMap<FixtureId, Fixture> {
        &self.fixtures
    }
}
