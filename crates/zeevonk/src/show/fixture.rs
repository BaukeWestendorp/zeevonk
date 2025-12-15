//! Fixture definitions and builders used by GDCS.

use std::collections::HashMap;
use std::num::NonZeroU32;
use std::{cmp, fmt, str};

use uuid::Uuid;

use crate::Error;
use crate::attr::Attribute;
use crate::dmx::Address;
use crate::value::ClampedValue;

/// A configured fixture instance.
#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Fixture {
    pub(crate) path: FixturePath,
    pub(crate) root_base_address: Address,
    pub(crate) name: String,

    pub(crate) gdtf_fixture_type_id: Uuid,
    pub(crate) gdtf_dmx_mode: String,
    pub(crate) channel_functions: HashMap<Attribute, FixtureChannelFunction>,

    pub(crate) sub_fixture_paths: Vec<FixturePath>,
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
    pub fn channel_functions(&self) -> impl Iterator<Item = (&Attribute, &FixtureChannelFunction)> {
        self.channel_functions.iter()
    }
}

/// Describes how a fixture attribute maps to DMX channel values.
///
/// A channel function defines whether the attribute is controlled by
/// physical DMX addresses or derived virtually from other attributes,
/// and the range of values it accepts (min/max) and its default value.
#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Relation {
    pub(crate) kind: RelationKind,
    pub(crate) fixture_path: FixturePath,
    pub(crate) attribute: Attribute,
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
    pub fn attribute(&self) -> Attribute {
        self.attribute
    }
}

/// The operation used when combining a source attribute into a virtual attribute.
#[derive(Debug, Clone, Copy)]
#[derive(serde::Serialize, serde::Deserialize)]
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
    /// Returns `Err(Error::InvalidFixtureId)` if `id` is zero.
    pub fn new(id: u32) -> Result<Self, Error> {
        match NonZeroU32::new(id) {
            Some(id) => Ok(FixtureId(id)),
            None => Err(Error::other(format!("non-zero fixture id: {id}"))),
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
    pub fn offset(self, offset: i32) -> Result<Self, Error> {
        let id = self.as_u32() as i32 + offset;
        match NonZeroU32::new(id as u32) {
            Some(id) => Ok(FixtureId(id)),

            None => Err(Error::other(format!("invalid fixture id: {id}"))),
        }
    }
}

impl fmt::Display for FixtureId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_u32())
    }
}

impl str::FromStr for FixtureId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let id = s.parse::<u32>().map_err(|_| Error::other(format!("non-zero fixture id: 0")))?;
        FixtureId::new(id)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
/// A path of [FixtureId] values.
///
/// The first element is considered the "root" fixture and additional
/// elements are sub-fixtures. The maximum number of elements is [FixturePath::MAX_LEN].
pub struct FixturePath {
    ids: [FixtureId; Self::MAX_LEN],
    len: u8,
}

impl FixturePath {
    /// Maximum number of [FixtureId]s that can be stored in a [FixturePath].
    pub const MAX_LEN: usize = 8;

    /// Create a new [FixturePath] containing only the given root fixture.
    pub fn new(root_id: FixtureId) -> Self {
        let mut ids = [FixtureId::new(1).unwrap(); Self::MAX_LEN];
        ids[0] = root_id;
        FixturePath { ids, len: 1 }
    }

    /// Append a fixture identifier to the end of the path.
    ///
    /// # Panics
    ///
    /// Panics if the path already contains [FixturePath::MAX_LEN] elements.
    pub fn push(&mut self, id: FixtureId) {
        let len = self.len();
        assert!(len < Self::MAX_LEN, "FixturePath capacity exceeded (max {})", Self::MAX_LEN);
        self.ids[len] = id;
        self.len = (len + 1) as u8;
    }

    /// Returns the number of fixtures in this path.
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns `true` if this path contains only the root fixture.
    pub fn is_root_fixture(&self) -> bool {
        self.len == 1
    }

    /// Returns the number of sub-fixtures (excluding the root).
    pub fn sub_len(&self) -> usize {
        assert!(self.len() >= 1, "FixturePath must have at least a root");
        self.len() - 1
    }

    /// Returns the root [FixtureId] of the path.
    pub fn root(&self) -> FixtureId {
        self.ids[0]
    }

    /// Returns the last [FixtureId] in the path.
    pub fn last(&self) -> FixtureId {
        let l = self.len();
        assert!(l >= 1, "FixturePath must have at least a root");
        self.ids[l - 1]
    }

    /// Borrow the path as a slice of [FixtureId]s.
    pub fn as_slice(&self) -> &[FixtureId] {
        &self.ids[..self.len()]
    }

    /// Returns an iterator over the fixture identifiers in the path.
    pub fn iter(&self) -> std::slice::Iter<'_, FixtureId> {
        self.as_slice().iter()
    }

    /// Replace the last element of the path with `sub_id`.
    pub fn replace_last(&mut self, sub_id: FixtureId) {
        let l = self.len();
        assert!(l >= 1, "FixturePath must have at least a root");
        self.ids[l - 1] = sub_id;
    }

    /// Return a new [FixturePath] with `part` appended.
    pub fn extended_with(mut self, part: FixtureId) -> FixturePath {
        self.push(part);
        self
    }

    /// Returns `true` if `self` contains `path` as a prefix.
    pub fn contains(&self, path: &FixturePath) -> bool {
        let path_len = path.len();
        if path_len > self.len() {
            return false;
        }
        &self.as_slice()[..path_len] == path.as_slice()
    }
}

impl AsRef<[FixtureId]> for FixturePath {
    fn as_ref(&self) -> &[FixtureId] {
        self.as_slice()
    }
}

impl From<FixtureId> for FixturePath {
    fn from(id: FixtureId) -> Self {
        FixturePath::new(id)
    }
}

impl From<&[FixtureId]> for FixturePath {
    fn from(slice: &[FixtureId]) -> Self {
        assert!(
            slice.len() <= FixturePath::MAX_LEN,
            "FixturePath slice length {} exceeds capacity {}",
            slice.len(),
            FixturePath::MAX_LEN
        );
        let mut ids = [FixtureId::new(1).unwrap(); FixturePath::MAX_LEN];
        for (i, v) in slice.iter().enumerate() {
            ids[i] = *v;
        }
        FixturePath { ids, len: slice.len() as u8 }
    }
}

impl IntoIterator for FixturePath {
    type Item = FixtureId;
    type IntoIter = std::vec::IntoIter<FixtureId>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().to_vec().into_iter()
    }
}

impl<'a> IntoIterator for &'a FixturePath {
    type Item = &'a FixtureId;
    type IntoIter = std::slice::Iter<'a, FixtureId>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl cmp::PartialOrd for FixturePath {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for FixturePath {
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
impl fmt::Display for FixturePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for id in self.as_slice() {
            if !first {
                write!(f, ".")?;
            }
            write!(f, "{}", id)?;
            first = false;
        }
        Ok(())
    }
}

impl fmt::Debug for FixturePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FixturePath(")?;
        fmt::Display::fmt(self, f)?;
        write!(f, ")")
    }
}

impl str::FromStr for FixturePath {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('.').collect();

        if parts.is_empty() {
            return Err(Error::other("empty fixture path"));
        }

        if parts.len() > FixturePath::MAX_LEN {
            return Err(Error::other(format!(
                "fixture path has too many parts (max {})",
                FixturePath::MAX_LEN,
            )));
        }
        let mut ids = [FixtureId::new(1).unwrap(); FixturePath::MAX_LEN];
        for (i, part) in parts.iter().enumerate() {
            ids[i] = FixtureId::from_str(part)?;
        }
        Ok(FixturePath { ids, len: parts.len() as u8 })
    }
}

impl serde::Serialize for FixturePath {
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

impl<'de> serde::Deserialize<'de> for FixturePath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct FixturePathVisitor;

        impl<'de> serde::de::Visitor<'de> for FixturePathVisitor {
            type Value = FixturePath;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string representing a FixturePath")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                use std::str::FromStr;
                FixturePath::from_str(v).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(FixturePathVisitor)
    }
}

#[macro_export]
macro_rules! fpath {
    ( $first:literal $(, $rest:literal )* $(,)? ) => {{
        let mut p = $crate::show::fixture::FixturePath::new(
            $crate::show::fixture::FixtureId::new($first).unwrap()
        );
        $( p.push($crate::show::fixture::FixtureId::new($rest).unwrap()); )*
        p
    }};
    ( $first:expr $(, $rest:expr )* $(,)? ) => {{
        let mut p = $crate::fixture::FixturePath::new($first);
        $( p.push($rest); )*
        p
    }};
}
