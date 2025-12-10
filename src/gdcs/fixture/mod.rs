use std::collections::HashMap;
use std::num::NonZeroU32;
use std::{fmt, str};

use uuid::Uuid;

pub use path::FixturePath;

use crate::dmx::Address;
use crate::gdcs::attr::Attribute;
use crate::gdcs::{ClampedValue, GdcsError};

pub(crate) mod builder;
pub mod path;

#[derive(Debug)]
pub struct Fixture {
    pub(super) path: FixturePath,
    pub(super) root_base_address: Address,
    pub(super) name: String,

    pub(super) gdtf_fixture_type_id: Uuid,
    pub(super) gdtf_dmx_mode: String,
    pub(super) channel_functions: HashMap<Attribute, FixtureChannelFunction>,

    pub(super) sub_fixture_paths: Vec<FixturePath>,
}

impl Fixture {
    pub fn path(&self) -> FixturePath {
        self.path
    }

    pub fn base_address(&self) -> Address {
        self.root_base_address
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn sub_fixtures(&self) -> &[FixturePath] {
        &self.sub_fixture_paths
    }

    pub fn gdtf_fixture_type_id(&self) -> Uuid {
        self.gdtf_fixture_type_id
    }

    pub fn gdtf_dmx_mode(&self) -> &str {
        &self.gdtf_dmx_mode
    }

    pub fn channel_function(&self, attribute: &Attribute) -> Option<&FixtureChannelFunction> {
        self.channel_functions.get(attribute)
    }

    pub fn channel_functions(
        &self,
    ) -> impl IntoIterator<Item = (&Attribute, &FixtureChannelFunction)> {
        self.channel_functions.iter()
    }
}

#[derive(Debug)]
pub struct FixtureChannelFunction {
    kind: FixtureChannelFunctionKind,
    from: ClampedValue,
    to: ClampedValue,
    default: ClampedValue,
}

impl FixtureChannelFunction {
    pub(crate) fn new(
        kind: FixtureChannelFunctionKind,
        from: ClampedValue,
        to: ClampedValue,
        default: ClampedValue,
    ) -> Self {
        Self { kind, from, to, default }
    }

    pub fn kind(&self) -> &FixtureChannelFunctionKind {
        &self.kind
    }

    pub fn from(&self) -> ClampedValue {
        self.from
    }

    pub fn to(&self) -> ClampedValue {
        self.to
    }

    pub fn default(&self) -> ClampedValue {
        self.default
    }
}

#[derive(Debug)]
pub enum FixtureChannelFunctionKind {
    Physical { addresses: Vec<Address> },
    Virtual { relations: Vec<Relation> },
}

#[derive(Debug)]
pub struct Relation {
    kind: RelationKind,
    fixture_path: FixturePath,
    attribute: Attribute,
}

impl Relation {
    pub fn new(kind: RelationKind, fixture_path: FixturePath, attribute: Attribute) -> Self {
        Self { kind, fixture_path, attribute }
    }

    pub fn kind(&self) -> &RelationKind {
        &self.kind
    }

    pub fn fixture_path(&self) -> FixturePath {
        self.fixture_path
    }

    pub fn attribute(&self) -> &Attribute {
        &self.attribute
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RelationKind {
    Multiply,
    Override,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct FixtureId(NonZeroU32);

impl FixtureId {
    pub fn new(id: u32) -> Result<Self, GdcsError> {
        match NonZeroU32::new(id) {
            Some(id) => Ok(FixtureId(id)),
            None => Err(GdcsError::InvalidFixtureId(id)),
        }
    }

    pub fn as_u32(&self) -> u32 {
        self.0.into()
    }

    pub fn offset(self, offset: i32) -> Result<Self, GdcsError> {
        let id = self.as_u32() as i32 + offset;
        match NonZeroU32::new(id as u32) {
            Some(id) => Ok(FixtureId(id)),
            None => Err(GdcsError::InvalidFixtureId(id as u32)),
        }
    }
}

impl fmt::Display for FixtureId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_u32())
    }
}

impl str::FromStr for FixtureId {
    type Err = GdcsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse::<u32>().map_err(|_| GdcsError::InvalidFixtureId(0))?;
        FixtureId::new(id)
    }
}
