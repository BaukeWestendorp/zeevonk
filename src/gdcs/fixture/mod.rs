//! Fixture definitions and builders used by GDCS.

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::{fmt, str};

use uuid::Uuid;

pub use path::*;

use crate::dmx::Address;
use crate::gdcs::attr::Attribute;
use crate::gdcs::{ClampedValue, GdcsError};

pub(crate) mod builder;
mod path;

/// A configured fixture instance.
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
    /// Returns the path identifying this fixture within the fixture tree.
    pub fn path(&self) -> FixturePath {
        self.path
    }

    /// Returns the root DMX base address assigned to this fixture.
    ///
    /// This is the first address occupied by the fixture in the DMX
    /// universe (addresses occupied by sub-fixtures are derived from this).
    pub fn base_address(&self) -> Address {
        self.root_base_address
    }

    /// Returns the name for the fixture instance.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the paths of any sub-fixtures contained by this fixture.
    pub fn sub_fixtures(&self) -> &[FixturePath] {
        &self.sub_fixture_paths
    }

    /// Returns the GDTF fixture type this instance is based on.
    pub fn gdtf_fixture_type_id(&self) -> Uuid {
        self.gdtf_fixture_type_id
    }

    /// Returns the GDTF DMX mode used by this fixture instance.
    pub fn gdtf_dmx_mode(&self) -> &str {
        &self.gdtf_dmx_mode
    }

    /// Get the channel function associated with the given attribute.
    ///
    /// Returns `None` if the attribute is not present on this fixture.
    pub fn channel_function(&self, attribute: &Attribute) -> Option<&FixtureChannelFunction> {
        self.channel_functions.get(attribute)
    }

    /// Get all channel functions for this fixture.
    pub fn channel_functions(
        &self,
    ) -> impl IntoIterator<Item = (&Attribute, &FixtureChannelFunction)> {
        self.channel_functions.iter()
    }
}

/// Describes how a fixture attribute maps to DMX channel values.
///
/// A channel function defines whether the attribute is controlled by
/// physical DMX addresses or derived virtually from other attributes,
/// and the range of values it accepts (from/to) and its default value.
#[derive(Debug)]
pub struct FixtureChannelFunction {
    kind: FixtureChannelFunctionKind,
    from: ClampedValue,
    to: ClampedValue,
    default: ClampedValue,
}

impl FixtureChannelFunction {
    /// Returns the kind of this channel function (physical or virtual).
    pub fn kind(&self) -> &FixtureChannelFunctionKind {
        &self.kind
    }

    /// The minimum value (inclusive) supported by this channel function.
    pub fn from(&self) -> ClampedValue {
        self.from
    }

    /// The maximum value (inclusive) supported by this channel function.
    pub fn to(&self) -> ClampedValue {
        self.to
    }

    /// The default value for this attribute when no explicit value is set.
    pub fn default(&self) -> ClampedValue {
        self.default
    }
}

/// Specifies whether an attribute is mapped to physical DMX channels or is
/// computed virtually from other attributes.
#[derive(Debug)]
pub enum FixtureChannelFunctionKind {
    /// A physical channel mapping addresses to a channel functions.
    /// (multiple are used for fine-controlled channel functions like Pan or Tilt).
    Physical {
        /// DMX addresses.
        addresses: Vec<Address>,
    },

    /// A virtual mapping derived from relationships to other fixture attributes.
    Virtual {
        /// Relations to other fixture attributes used to compute the value.
        relations: Vec<Relation>,
    },
}

/// A relation describes how a virtual attribute is derived from another
/// attribute.
#[derive(Debug)]
pub struct Relation {
    kind: RelationKind,
    fixture_path: FixturePath,
    attribute: Attribute,
}

impl Relation {
    /// Creates a new `Relation`.
    pub fn new(kind: RelationKind, fixture_path: FixturePath, attribute: Attribute) -> Self {
        Self { kind, fixture_path, attribute }
    }

    /// Returns the relation kind (e.g. multiply or override).
    pub fn kind(&self) -> &RelationKind {
        &self.kind
    }

    /// Returns the path to the fixture this relation references.
    pub fn fixture_path(&self) -> FixturePath {
        self.fixture_path
    }

    /// Returns the attribute on the referenced fixture used by this relation.
    pub fn attribute(&self) -> &Attribute {
        &self.attribute
    }
}

/// The operation used when combining a source attribute into a virtual attribute.
#[derive(Debug, Clone, Copy)]
pub enum RelationKind {
    /// Multiply the source attribute value with the target.
    Multiply,
    /// Override the target with the source attribute value.
    Override,
}

/// A non-zero identifier for a fixture.
///
/// `FixtureId` guarantees the inner identifier is never zero. Use
/// `FixtureId::new` to construct a validated id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct FixtureId(NonZeroU32);

impl FixtureId {
    /// Create a new `FixtureId` from a raw `u32`.
    ///
    /// Returns `Err(GdcsError::InvalidFixtureId)` if `id` is zero.
    pub fn new(id: u32) -> Result<Self, GdcsError> {
        match NonZeroU32::new(id) {
            Some(id) => Ok(FixtureId(id)),
            None => Err(GdcsError::InvalidFixtureId(id)),
        }
    }

    /// Return the underlying identifier as a `u32`.
    pub fn as_u32(&self) -> u32 {
        self.0.into()
    }

    /// Return a new `FixtureId` offset by the given signed integer.
    ///
    /// Useful for computing adjacent fixture identifiers. Returns an error
    /// if the resulting id would be zero or otherwise invalid.
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
