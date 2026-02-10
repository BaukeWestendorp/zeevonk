//! Baked information about each fixture and their children,
//! including attributes and their channels.

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::{cmp, fmt, str};

use crate::theymx::{Address, Multiverse};
use uuid::Uuid;

use crate::Error;
use crate::attr::Attribute;
use crate::value::ClampedValue;

/// Represents a stage containing all fixtures and their configuration.
#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Stage {
    /// The defaulted multiverse used for address resolution.
    pub(crate) defaulted_multiverse: Multiverse,

    /// Map of all fixtures in this stage, keyed by their [`FixtureId`].
    pub(crate) fixtures: HashMap<FixtureId, Fixture>,
}

impl Stage {
    /// Returns a reference to the defaulted [`Multiverse`] used for address resolution.
    pub fn defaulted_multiverse(&self) -> &Multiverse {
        &self.defaulted_multiverse
    }

    /// Returns the map of fixtures contained in this stage.
    pub fn fixtures(&self) -> &HashMap<FixtureId, Fixture> {
        &self.fixtures
    }

    /// Returns an iterator over all root fixtures in this stage.
    pub fn root_fixtures(&self) -> impl Iterator<Item = (&FixtureId, &Fixture)> {
        self.fixtures.iter().filter(|(_, fixture)| fixture.id.is_root())
    }

    /// Returns a reference to the fixture with the given [`FixtureId`], if present.
    pub fn fixture(&self, id: &FixtureId) -> Option<&Fixture> {
        self.fixtures.get(id)
    }

    /// Returns an iterator over all direct children of the given fixture.
    pub fn child_fixtures(&self, id: &FixtureId) -> impl Iterator<Item = (&FixtureId, &Fixture)> {
        self.fixtures.iter().filter(move |(fid, _)| {
            fid.len() > id.len() && fid.as_slice()[..id.len()] == *id.as_slice()
        })
    }

    /// Returns an iterator over all descendant fixtures of the fixture with the given [`FixtureId`].
    ///
    /// A *descendant* is any fixture whose identifier has `id` as a prefix, excluding `id` itself.
    /// This includes both direct children and deeper nested sub-fixtures.
    pub fn descendant_fixtures(
        &self,
        id: &FixtureId,
    ) -> impl Iterator<Item = (&FixtureId, &Fixture)> {
        self.fixtures.iter().filter(move |(fid, _)| id.contains(fid) && *fid != id)
    }

    /// Returns true if the stage contains a fixture with the given [`FixtureId`].
    pub fn contains_fixture(&self, id: &FixtureId) -> bool {
        self.fixtures.contains_key(id)
    }

    /// Returns the total number of fixtures in this stage.
    pub fn fixture_count(&self) -> usize {
        self.fixtures.len()
    }
}

/// A configured fixture instance.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Fixture {
    pub(crate) id: FixtureId,
    pub(crate) root_base_address: Address,
    pub(crate) name: String,

    pub(crate) gdtf_fixture_type_id: Uuid,
    pub(crate) gdtf_dmx_mode: String,
    pub(crate) channel_functions: HashMap<Attribute, FixtureChannelFunction>,

    pub(crate) child_ids: Vec<FixtureId>,
}

impl Fixture {
    /// Returns the identifier identifying this fixture within the fixture tree.
    pub fn id(&self) -> FixtureId {
        self.id
    }

    /// Returns the root DMX base address assigned to this fixture.
    ///
    /// This is the first address occupied by the fixture in the DMX
    /// universe (addresses occupied by descendant fixtures are derived from this).
    pub fn base_address(&self) -> Address {
        self.root_base_address
    }

    /// Returns the name for the fixture instance.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the identifiers of any children contained by this fixture.
    pub fn child_ids(&self) -> &[FixtureId] {
        &self.child_ids
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
    pub fn channel_functions(&self) -> impl Iterator<Item = (&Attribute, &FixtureChannelFunction)> {
        self.channel_functions.iter()
    }
}

/// Describes how a fixture attribute maps to DMX channel values.
///
/// A channel function defines whether the attribute is controlled by
/// physical DMX addresses or derived virtually from other attributes,
/// and the range of values it accepts (min/max) and its default value.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FixtureChannelFunction {
    pub(crate) kind: FixtureChannelFunctionKind,
    pub(crate) min: ClampedValue,
    pub(crate) max: ClampedValue,
    pub(crate) default: ClampedValue,
}

impl FixtureChannelFunction {
    /// Returns the kind of this channel function (physical or virtual).
    pub fn kind(&self) -> &FixtureChannelFunctionKind {
        &self.kind
    }

    /// The minimum value (inclusive) supported by this channel function.
    pub fn min(&self) -> ClampedValue {
        self.min
    }

    /// The maximum value (inclusive) supported by this channel function.
    pub fn max(&self) -> ClampedValue {
        self.max
    }

    /// The default value for this attribute when no explicit value is set.
    pub fn default(&self) -> ClampedValue {
        self.default
    }
}

/// Specifies whether an attribute is mapped to physical DMX channels or is
/// computed virtually from other attributes.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Relation {
    pub(crate) kind: RelationKind,
    pub(crate) fixture_id: FixtureId,
    pub(crate) attribute: Attribute,
}

impl Relation {
    /// Creates a new [`Relation`].
    pub fn new(kind: RelationKind, fixture_id: FixtureId, attribute: Attribute) -> Self {
        Self { kind, fixture_id, attribute }
    }

    /// Returns the relation kind (e.g. multiply or override).
    pub fn kind(&self) -> &RelationKind {
        &self.kind
    }

    /// Returns the identifier to the fixture this relation references.
    pub fn fixture_id(&self) -> FixtureId {
        self.fixture_id
    }

    /// Returns the attribute on the referenced fixture used by this relation.
    pub fn attribute(&self) -> Attribute {
        self.attribute
    }
}

/// The operation used when combining a source attribute into a virtual attribute.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum RelationKind {
    /// Multiply the source attribute value with the target.
    Multiply,
    /// Override the target with the source attribute value.
    Override,
}

/// A non-zero identifier part for a fixture.
///
/// [`FixtureIdPart`] guarantees the inner identifier is never zero. Use
/// [`FixtureIdPart::new`] to construct a validated part.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct FixtureIdPart(NonZeroU32);

impl FixtureIdPart {
    /// Create a new [`FixtureIdPart`] from a raw `u32`.
    ///
    /// Returns [`Error::InvalidFixtureId`] if `id` is zero.
    pub fn new(id: u32) -> Result<Self, Error> {
        match NonZeroU32::new(id) {
            Some(id) => Ok(FixtureIdPart(id)),
            None => Err(Error::InvalidFixtureId),
        }
    }

    /// Return the underlying identifier as a `u32`.
    pub fn as_u32(&self) -> u32 {
        self.0.into()
    }

    /// Return a new [`FixtureIdPart`] offset by the given signed integer.
    ///
    /// Useful for computing adjacent fixture identifier parts. Returns an error
    /// if the resulting part would be zero or otherwise invalid.
    pub fn offset(self, offset: i32) -> Result<Self, Error> {
        let id = self.as_u32() as i32 + offset;
        match NonZeroU32::new(id as u32) {
            Some(id) => Ok(FixtureIdPart(id)),
            None => Err(Error::InvalidFixtureId),
        }
    }
}

impl fmt::Display for FixtureIdPart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_u32())
    }
}

impl str::FromStr for FixtureIdPart {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse::<u32>().map_err(|_| Error::InvalidFixtureId)?;
        FixtureIdPart::new(id)
    }
}

/// A composed fixture identifier made up of multiple [`FixtureIdPart`]s.
///
/// The first element is considered the "root" fixture and additional
/// elements are sub-fixtures. The maximum number of elements is [`FixtureId::MAX_LEN`].
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct FixtureId {
    ids: [FixtureIdPart; Self::MAX_LEN],
    len: u8,
}

impl FixtureId {
    /// Maximum number of [`FixtureIdPart`]s that can be stored in a [`FixtureId`].
    pub const MAX_LEN: usize = 8;

    /// Create a new [`FixtureId`] containing only the given root part.
    pub fn new(root_part: FixtureIdPart) -> Self {
        let mut ids = [FixtureIdPart::new(1).unwrap(); Self::MAX_LEN];
        ids[0] = root_part;
        FixtureId { ids, len: 1 }
    }

    /// Append a fixture identifier part to the end of the identifier.
    ///
    /// # Panics
    ///
    /// Panics if the identifier already contains [`FixtureId::MAX_LEN`] elements.
    pub fn push(&mut self, part: FixtureIdPart) {
        let len = self.len();
        assert!(len < Self::MAX_LEN, "FixtureId capacity exceeded (max {})", Self::MAX_LEN);
        self.ids[len] = part;
        self.len = (len + 1) as u8;
    }

    /// Returns the number of parts in this identifier.
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns `true` if the identifier contains no parts.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns `true` if this identifier contains only the root part.
    pub fn is_root(&self) -> bool {
        self.len == 1
    }

    /// Returns the number of sub-parts (excluding the root).
    pub fn sub_len(&self) -> usize {
        assert!(!self.is_empty(), "FixtureId must have at least a root");
        self.len() - 1
    }

    /// Returns the root [`FixtureIdPart`] of the identifier.
    pub fn root(&self) -> FixtureIdPart {
        self.ids[0]
    }

    /// Returns the last [`FixtureIdPart`] in the identifier.
    pub fn last(&self) -> FixtureIdPart {
        let l = self.len();
        assert!(l >= 1, "FixtureId must have at least a root");
        self.ids[l - 1]
    }

    /// Borrow the identifier as a slice of [`FixtureIdPart`]s.
    pub fn as_slice(&self) -> &[FixtureIdPart] {
        &self.ids[..self.len()]
    }

    /// Returns an iterator over the fixture identifier parts.
    pub fn iter(&self) -> std::slice::Iter<'_, FixtureIdPart> {
        self.as_slice().iter()
    }

    /// Replace the last element of the identifier.
    pub fn replace_last(&mut self, sub_part: FixtureIdPart) {
        let l = self.len();
        assert!(l >= 1, "FixtureId must have at least a root");
        self.ids[l - 1] = sub_part;
    }

    /// Return a new [`FixtureId`] with `part` appended.
    pub fn extended_with(mut self, part: FixtureIdPart) -> FixtureId {
        self.push(part);
        self
    }

    /// Returns `true` if the given fixture id is a subset (child).
    pub fn contains(&self, other: &FixtureId) -> bool {
        let other_len = other.len();
        if other_len > self.len() {
            return false;
        }
        &self.as_slice()[..other_len] == other.as_slice()
    }
}

impl AsRef<[FixtureIdPart]> for FixtureId {
    fn as_ref(&self) -> &[FixtureIdPart] {
        self.as_slice()
    }
}

impl From<FixtureIdPart> for FixtureId {
    fn from(part: FixtureIdPart) -> Self {
        FixtureId::new(part)
    }
}

impl From<&[FixtureIdPart]> for FixtureId {
    fn from(slice: &[FixtureIdPart]) -> Self {
        assert!(
            slice.len() <= FixtureId::MAX_LEN,
            "FixtureId slice length {} exceeds capacity {}",
            slice.len(),
            FixtureId::MAX_LEN
        );
        let mut ids = [FixtureIdPart::new(1).unwrap(); FixtureId::MAX_LEN];
        for (i, v) in slice.iter().enumerate() {
            ids[i] = *v;
        }
        FixtureId { ids, len: slice.len() as u8 }
    }
}

impl IntoIterator for FixtureId {
    type Item = FixtureIdPart;
    type IntoIter = std::vec::IntoIter<FixtureIdPart>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().to_vec().into_iter()
    }
}

impl<'a> IntoIterator for &'a FixtureId {
    type Item = &'a FixtureIdPart;
    type IntoIter = std::slice::Iter<'a, FixtureIdPart>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl cmp::PartialOrd for FixtureId {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for FixtureId {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        let a = self.as_slice();
        let b = other.as_slice();
        for (x, y) in a.iter().zip(b.iter()) {
            let ord = x.cmp(y);
            if ord != cmp::Ordering::Equal {
                return ord;
            }
        }
        a.len().cmp(&b.len())
    }
}

impl fmt::Display for FixtureId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for part in self.as_slice() {
            if !first {
                write!(f, ".")?;
            }
            write!(f, "{}", part)?;
            first = false;
        }
        Ok(())
    }
}

impl fmt::Debug for FixtureId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FixtureId(")?;
        fmt::Display::fmt(self, f)?;
        write!(f, ")")
    }
}

impl str::FromStr for FixtureId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();

        if parts.is_empty() {
            return Err(Error::EmptyFixtureId);
        }

        if parts.len() > FixtureId::MAX_LEN {
            return Err(Error::FixtureIdTooLong(FixtureId::MAX_LEN));
        }
        let mut ids = [FixtureIdPart::new(1).unwrap(); FixtureId::MAX_LEN];
        for (i, part) in parts.iter().enumerate() {
            ids[i] = FixtureIdPart::from_str(part)?;
        }
        Ok(FixtureId { ids, len: parts.len() as u8 })
    }
}

impl serde::Serialize for FixtureId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use std::fmt::Write;
        let mut s = String::new();
        write!(&mut s, "{}", self).unwrap();
        serializer.serialize_str(&s)
    }
}

impl<'de> serde::Deserialize<'de> for FixtureId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct FixtureIdVisitor;

        impl<'de> serde::de::Visitor<'de> for FixtureIdVisitor {
            type Value = FixtureId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string representing a FixtureId")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::str::FromStr;
                FixtureId::from_str(v).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(FixtureIdVisitor)
    }
}
