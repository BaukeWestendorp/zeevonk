use std::str;
use uuid::Uuid;

use crate::dmx::Address;
use crate::state::fixture::FixtureId;

/// A patch containing a list of [`Fixture`]s.
#[derive(Debug, Clone, PartialEq, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Patch {
    fixtures: Vec<Fixture>,
}

impl Patch {
    /// Returns all fixtures in the [`Patch`].
    pub fn fixtures(&self) -> &[Fixture] {
        &self.fixtures
    }
}

/// A single fixture in the [`Patch`].
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Fixture {
    id: FixtureId,
    label: String,
    address: Address,
    kind: FixtureKind,
}

impl Fixture {
    /// Returns the unique [`FixtureId`] of the fixture.
    pub fn id(&self) -> FixtureId {
        self.id
    }

    /// Returns the label of the fixture.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the DMX [`Address`] of the fixture.
    pub fn address(&self) -> Address {
        self.address
    }

    /// Returns the [`FixtureKind`] of the fixture.
    pub fn kind(&self) -> &FixtureKind {
        &self.kind
    }
}

/// Describes the GDTF fixture type and DMX mode of a [`Fixture`].
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct FixtureKind {
    gdtf_fixture_type_id: Uuid,
    gdtf_dmx_mode: String,
}

impl FixtureKind {
    /// Returns the [`Uuid`] of the GDTF fixture type.
    pub fn gdtf_fixture_type_id(&self) -> Uuid {
        self.gdtf_fixture_type_id
    }

    /// Returns the DMX mode of the fixture.
    pub fn gdtf_dmx_mode(&self) -> &str {
        &self.gdtf_dmx_mode
    }
}
