use std::num::NonZeroU32;
use std::{fmt, str};

use anyhow::{Context, bail};
use uuid::Uuid;

use crate::dmx::Address;
use crate::showfile::Label;

#[derive(Debug, Clone, PartialEq, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Patch {
    fixtures: Vec<Fixture>,
}

impl Patch {
    pub fn fixtures(&self) -> &[Fixture] {
        &self.fixtures
    }
}

#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Fixture {
    id: FixtureId,
    label: Label,
    address: Address,
    kind: FixtureKind,
}

impl Fixture {
    pub fn id(&self) -> FixtureId {
        self.id
    }

    pub fn label(&self) -> &Label {
        &self.label
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub fn kind(&self) -> &FixtureKind {
        &self.kind
    }
}

#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct FixtureKind {
    fixture_type_id: Uuid,
    dmx_mode: String,
}

impl FixtureKind {
    pub fn fixture_type_id(&self) -> Uuid {
        self.fixture_type_id
    }

    pub fn dmx_mode(&self) -> &str {
        &self.dmx_mode
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct FixtureId(NonZeroU32);

impl FixtureId {
    /// Create a new FixtureId. Returns an error if id is zero.
    pub fn new(id: u32) -> anyhow::Result<Self> {
        match NonZeroU32::new(id) {
            Some(id) => Ok(FixtureId(id)),
            None => bail!("FixtureId must be non-zero (got {})", id),
        }
    }

    pub fn as_u32(&self) -> u32 {
        self.0.into()
    }

    /// Returns a new FixtureId offset by the given value. Returns an error if the result is zero or negative.
    pub fn offset(self, offset: i32) -> anyhow::Result<Self> {
        let id = self.as_u32() as i32 + offset;

        if id <= 0 {
            bail!(
                "offsetting FixtureId {} by {} results in invalid id {} (must be non-zero and positive)",
                self.as_u32(),
                offset,
                id
            );
        }

        match NonZeroU32::new(id as u32) {
            Some(id) => Ok(FixtureId(id)),
            None => bail!(
                "offsetting FixtureId {} by {} results in invalid id {} (must be non-zero)",
                self.as_u32(),
                offset,
                id
            ),
        }
    }
}

impl fmt::Display for FixtureId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_u32())
    }
}

impl str::FromStr for FixtureId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s
            .parse::<u32>()
            .with_context(|| format!("failed to parse FixtureId from '{}': not a valid u32", s))?;
        FixtureId::new(id)
            .with_context(|| format!("fixtureId parsed from '{}' must be non-zero", s))
    }
}
