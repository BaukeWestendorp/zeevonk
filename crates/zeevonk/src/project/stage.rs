//! Baked information about each fixture and their children,
//! including attributes and their channels.

use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::{cmp, fmt, str};

use rigger::gdtf::attr::AttributeName;
use uuid::Uuid;

use crate::Error;
use crate::theymx::Address;
use crate::value::AttributeValues;

/// A read-only, "baked" view of a patch that contains
/// fixtures and their configuration.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Stage {
    /// Default attribute values for fixtures in this stage.
    pub(crate) default_attribute_values: AttributeValues,

    /// Map of all fixtures in this stage, keyed by their [`FixtureId`].
    pub(crate) fixtures: BTreeMap<FixtureId, Fixture>,
}

impl Stage {
    /// Returns the default attribute values for fixtures in this stage.
    pub fn default_attribute_values(&self) -> &AttributeValues {
        &self.default_attribute_values
    }

    /// Returns the map of fixtures contained in this stage.
    pub fn fixtures(&self) -> &BTreeMap<FixtureId, Fixture> {
        &self.fixtures
    }

    /// Returns `true` if the stage contains no fixtures.
    pub fn is_empty(&self) -> bool {
        self.fixtures.is_empty()
    }

    /// Returns the total number of fixtures in this stage.
    pub fn len(&self) -> usize {
        self.fixtures.len()
    }

    /// Returns a reference to the fixture with the given [`FixtureId`], if present.
    pub fn get(&self, id: &FixtureId) -> Option<&Fixture> {
        self.fixtures.get(id)
    }

    /// Backwards-compatible name for [`Stage::get`].
    pub fn fixture(&self, id: &FixtureId) -> Option<&Fixture> {
        self.get(id)
    }

    /// Returns true if the stage contains a fixture with the given [`FixtureId`].
    pub fn contains_fixture(&self, id: &FixtureId) -> bool {
        self.fixtures.contains_key(id)
    }

    /// Returns an iterator over all root fixtures in this stage.
    pub fn roots(&self) -> impl Iterator<Item = (&FixtureId, &Fixture)> {
        self.fixtures.iter().filter(|(id, _)| id.is_root())
    }

    /// Returns an iterator over all *direct* children of the given fixture.
    ///
    /// A direct child has exactly one more [`FixtureIdPart`] than `parent`.
    pub fn children(&self, parent: &FixtureId) -> impl Iterator<Item = (&FixtureId, &Fixture)> {
        let parent_len = parent.len();
        self.fixtures
            .iter()
            .filter(move |(id, _)| id.len() == parent_len + 1 && id.contains(parent))
    }

    /// Returns an iterator over all descendant fixtures of `ancestor` (excluding `ancestor`).
    ///
    /// A *descendant* is any fixture whose identifier has `ancestor` as a prefix.
    pub fn descendants(
        &self,
        ancestor: &FixtureId,
    ) -> impl Iterator<Item = (&FixtureId, &Fixture)> {
        self.fixtures
            .range(ancestor.clone()..)
            .take_while(move |(id, _)| id.contains(ancestor))
            .filter(move |(id, _)| *id != ancestor)
    }

    /// Returns the parent id of `id`, if it is not a root identifier.
    pub fn parent_id(&self, id: &FixtureId) -> Option<FixtureId> {
        let slice = id.as_slice();
        if slice.len() <= 1 {
            return None;
        }
        Some(FixtureId::from(&slice[..slice.len() - 1]))
    }

    /// Returns the root id (the first identifier part) for any fixture id.
    pub fn root_id(&self, id: &FixtureId) -> FixtureId {
        FixtureId::from(id.root())
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
    pub(crate) channel_functions: BTreeMap<AttributeName, FixtureChannelFunction>,
    pub(crate) highlight_values: BTreeMap<Address, crate::theymx::Value>,

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
    pub fn channel_function(&self, attribute: &AttributeName) -> Option<&FixtureChannelFunction> {
        self.channel_functions.get(attribute)
    }

    /// Get all channel functions for this fixture.
    pub fn channel_functions(
        &self,
    ) -> impl Iterator<Item = (&AttributeName, &FixtureChannelFunction)> {
        self.channel_functions.iter()
    }

    /// Returns the "highlight" DMX values for this fixture.
    pub fn highlight_values(&self) -> &BTreeMap<Address, crate::theymx::Value> {
        &self.highlight_values
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
    pub(crate) min: f32,
    pub(crate) max: f32,
    pub(crate) default: f32,
}

impl FixtureChannelFunction {
    /// Returns the kind of this channel function (physical or virtual).
    pub fn kind(&self) -> &FixtureChannelFunctionKind {
        &self.kind
    }

    /// The minimum value (inclusive) supported by this channel function.
    pub fn min(&self) -> f32 {
        self.min
    }

    /// The maximum value (inclusive) supported by this channel function.
    pub fn max(&self) -> f32 {
        self.max
    }

    /// The default value for this attribute when no explicit value is set.
    pub fn default(&self) -> f32 {
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
    pub(crate) attribute: AttributeName,
}

impl Relation {
    /// Creates a new [`Relation`].
    pub fn new(kind: RelationKind, fixture_id: FixtureId, attribute: AttributeName) -> Self {
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
    pub fn attribute(&self) -> &AttributeName {
        &self.attribute
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
        let base = self.as_u32() as i64;
        let id = base + offset as i64;
        if id <= 0 || id > u32::MAX as i64 {
            return Err(Error::InvalidFixtureId);
        }
        Ok(FixtureIdPart(NonZeroU32::new(id as u32).unwrap()))
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
        self.try_push(part).expect("FixtureId capacity exceeded");
    }

    /// Append a fixture identifier part to the end of the identifier.
    ///
    /// Returns an error if the identifier already contains [`FixtureId::MAX_LEN`] elements.
    pub fn try_push(&mut self, part: FixtureIdPart) -> Result<(), Error> {
        let len = self.len();
        if len >= Self::MAX_LEN {
            return Err(Error::FixtureIdTooLong(Self::MAX_LEN));
        }
        self.ids[len] = part;
        self.len = (len + 1) as u8;
        Ok(())
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

    /// Returns `true` if `prefix` is a prefix (ancestor) of `self`.
    pub fn starts_with_fixture_id(&self, prefix: &FixtureId) -> bool {
        let prefix_len = prefix.len();
        if prefix_len > self.len() {
            return false;
        }
        &self.as_slice()[..prefix_len] == prefix.as_slice()
    }

    /// Returns `true` if the given fixture id is a prefix (ancestor) of `self`.
    pub fn contains(&self, other: &FixtureId) -> bool {
        self.starts_with_fixture_id(other)
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

/// [`FixtureId`] iterator.
pub struct FixtureIdIntoIter {
    id: FixtureId,
    idx: u8,
}

impl Iterator for FixtureIdIntoIter {
    type Item = FixtureIdPart;

    fn next(&mut self) -> Option<Self::Item> {
        let len = self.id.len as usize;
        let idx = self.idx as usize;
        if idx >= len {
            return None;
        }
        self.idx += 1;
        Some(self.id.ids[idx])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.id.len as usize).saturating_sub(self.idx as usize);
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for FixtureIdIntoIter {}

impl IntoIterator for FixtureId {
    type Item = FixtureIdPart;
    type IntoIter = FixtureIdIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        FixtureIdIntoIter { id: self, idx: 0 }
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
        if s.is_empty() {
            return Err(Error::EmptyFixtureId);
        }

        let mut ids = [FixtureIdPart::new(1).unwrap(); FixtureId::MAX_LEN];
        let mut len: usize = 0;

        for part in s.split('.') {
            if len >= FixtureId::MAX_LEN {
                return Err(Error::FixtureIdTooLong(FixtureId::MAX_LEN));
            }
            ids[len] = FixtureIdPart::from_str(part)?;
            len += 1;
        }

        if len == 0 {
            return Err(Error::EmptyFixtureId);
        }

        Ok(FixtureId { ids, len: len as u8 })
    }
}

impl serde::Serialize for FixtureId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
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

/// Helper trait to convert various types into a [`FixtureId`] more ergonomically.
pub trait IntoFixtureId {
    /// Returns `None` if the conversion fails.
    fn into_fixture_id(self) -> Option<FixtureId>;
}

impl IntoFixtureId for FixtureId {
    fn into_fixture_id(self) -> Option<FixtureId> {
        Some(self)
    }
}

impl IntoFixtureId for &FixtureId {
    fn into_fixture_id(self) -> Option<FixtureId> {
        Some(self.clone())
    }
}

impl IntoFixtureId for &str {
    fn into_fixture_id(self) -> Option<FixtureId> {
        self.parse().ok()
    }
}

impl IntoFixtureId for String {
    fn into_fixture_id(self) -> Option<FixtureId> {
        self.parse().ok()
    }
}

impl IntoFixtureId for u8 {
    fn into_fixture_id(self) -> Option<FixtureId> {
        FixtureIdPart::new(self as u32).ok().map(FixtureId::from)
    }
}

impl IntoFixtureId for i8 {
    fn into_fixture_id(self) -> Option<FixtureId> {
        if self > 0 { FixtureIdPart::new(self as u32).ok().map(FixtureId::from) } else { None }
    }
}

impl IntoFixtureId for u16 {
    fn into_fixture_id(self) -> Option<FixtureId> {
        FixtureIdPart::new(self as u32).ok().map(FixtureId::from)
    }
}

impl IntoFixtureId for i16 {
    fn into_fixture_id(self) -> Option<FixtureId> {
        if self > 0 { FixtureIdPart::new(self as u32).ok().map(FixtureId::from) } else { None }
    }
}

impl IntoFixtureId for u32 {
    fn into_fixture_id(self) -> Option<FixtureId> {
        FixtureIdPart::new(self).ok().map(FixtureId::from)
    }
}

impl IntoFixtureId for i32 {
    fn into_fixture_id(self) -> Option<FixtureId> {
        if self > 0 { FixtureIdPart::new(self as u32).ok().map(FixtureId::from) } else { None }
    }
}

impl IntoFixtureId for u64 {
    fn into_fixture_id(self) -> Option<FixtureId> {
        FixtureIdPart::new(self as u32).ok().map(FixtureId::from)
    }
}

impl IntoFixtureId for i64 {
    fn into_fixture_id(self) -> Option<FixtureId> {
        if self > 0 && self <= u32::MAX as i64 {
            FixtureIdPart::new(self as u32).ok().map(FixtureId::from)
        } else {
            None
        }
    }
}

impl IntoFixtureId for usize {
    fn into_fixture_id(self) -> Option<FixtureId> {
        if self > 0 && self <= u32::MAX as usize {
            FixtureIdPart::new(self as u32).ok().map(FixtureId::from)
        } else {
            None
        }
    }
}

/// Helper trait to convert various types into [`FixtureId`] sequences more ergonomically.
pub trait IntoFixtureIds {
    /// Returns an iterator of successfully converted [`FixtureId`]s.
    fn into_fixture_ids(self) -> Box<dyn Iterator<Item = FixtureId>>;
}

impl<'a, I, T> IntoFixtureIds for I
where
    I: IntoIterator<Item = T>,
    T: IntoFixtureId,
    <I as IntoIterator>::IntoIter: 'static,
{
    fn into_fixture_ids(self) -> Box<dyn Iterator<Item = FixtureId>> {
        Box::new(self.into_iter().filter_map(|item| item.into_fixture_id()))
    }
}

/// Specialized implementation for single FixtureId to support "one or many" APIs.
impl IntoFixtureIds for FixtureId {
    fn into_fixture_ids(self) -> Box<dyn Iterator<Item = FixtureId>> {
        Box::new(std::iter::once(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn part(n: u32) -> FixtureIdPart {
        FixtureIdPart::new(n).unwrap()
    }

    #[test]
    fn test_into_fixture_id_from_fixture_id() {
        let id = FixtureId::from(part(42));
        let result = id.clone().into_fixture_id();
        assert_eq!(result, Some(id));
    }

    #[test]
    fn test_into_fixture_id_from_ref_fixture_id() {
        let id = FixtureId::from(part(7));
        let result = (&id).into_fixture_id();
        assert_eq!(result, Some(id));
    }

    #[test]
    fn test_into_fixture_id_from_str_valid() {
        let s = "1.2.3";
        let id = FixtureId::from(&[part(1), part(2), part(3)][..]);
        let result = s.into_fixture_id();
        assert_eq!(result, Some(id));
    }

    #[test]
    fn test_into_fixture_id_from_str_invalid() {
        let s = "";
        let result = s.into_fixture_id();
        assert_eq!(result, None);

        let s = "0.2";
        let result = s.into_fixture_id();
        assert_eq!(result, None);

        let s = "1.2.3.4.5.6.7.8.9"; // Too long
        let result = s.into_fixture_id();
        assert_eq!(result, None);
    }

    #[test]
    fn test_into_fixture_id_from_string() {
        let s = String::from("5.6");
        let id = FixtureId::from(&[part(5), part(6)][..]);
        let result = s.into_fixture_id();
        assert_eq!(result, Some(id));
    }

    #[test]
    fn test_into_fixture_ids_from_vec_of_str() {
        let v = vec!["1.2", "3", "bad", "4.5.6"];
        let ids: Vec<_> = v.into_fixture_ids().collect();
        let expected = vec![
            FixtureId::from(&[part(1), part(2)][..]),
            FixtureId::from(part(3)),
            FixtureId::from(&[part(4), part(5), part(6)][..]),
        ];
        assert_eq!(ids, expected);
    }

    #[test]
    fn test_into_fixture_ids_from_vec_of_fixture_id() {
        let ids_in = vec![FixtureId::from(part(1)), FixtureId::from(&[part(2), part(3)][..])];
        let ids: Vec<_> = ids_in.clone().into_fixture_ids().collect();
        assert_eq!(ids, ids_in);
    }

    #[test]
    fn test_into_fixture_ids_from_vec_of_string() {
        let v = vec!["1.2".to_string(), "3".to_string()];
        let ids: Vec<_> = v.into_fixture_ids().collect();
        let expected = vec![FixtureId::from(&[part(1), part(2)][..]), FixtureId::from(part(3))];
        assert_eq!(ids, expected);
    }

    #[test]
    fn test_into_fixture_ids_from_fixture_id() {
        let id = FixtureId::from(&[part(9), part(8)][..]);
        let ids: Vec<_> = id.into_fixture_ids().collect();
        assert_eq!(ids, vec![FixtureId::from(&[part(9), part(8)][..])]);
    }

    #[test]
    fn test_into_fixture_ids_from_slice_of_str() {
        let arr = ["1", "2.3", "bad"];
        let ids: Vec<_> = arr.into_fixture_ids().collect();
        let expected = vec![FixtureId::from(part(1)), FixtureId::from(&[part(2), part(3)][..])];
        assert_eq!(ids, expected);
    }

    #[test]
    fn test_into_fixture_ids_from_slice_of_fixture_id() {
        let arr = [FixtureId::from(part(1)), FixtureId::from(&[part(2), part(3)][..])];
        let ids: Vec<_> = arr.into_fixture_ids().collect();
        assert_eq!(ids, arr);
    }
}
