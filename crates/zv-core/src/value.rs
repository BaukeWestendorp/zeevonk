//! Value types for clamped and mapped attribute values.

use std::collections::HashMap;
use std::{fmt, num, str};

use theymx::{self, Address};

use crate::attr::Attribute;
use crate::fixture::FixtureId;

/// A clamped value.
///
/// Represents a floating-point value constrained to the range
/// [0.0, 1.0]. All operations automatically clamp values to this valid range.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct ClampedValue(f32);

impl ClampedValue {
    /// The minimum allowed value (0.0).
    pub const MIN: f32 = 0.0;

    /// The maximum allowed value (1.0).
    pub const MAX: f32 = 1.0;

    /// Creates a new [`ClampedValue`] with the specified value.
    ///
    /// The value is automatically clamped to the range [0.0, 1.0].
    #[inline]
    pub const fn new(value: f32) -> Self {
        Self(value.clamp(Self::MIN, Self::MAX))
    }

    /// Sets the value of this [`ClampedValue`].
    ///
    /// The value is automatically clamped to the range [0.0, 1.0].
    #[inline]
    pub fn set(&mut self, value: f32) {
        self.0 = value.clamp(Self::MIN, Self::MAX);
    }

    /// Returns the underlying `f32` value.
    ///
    /// The returned value is guaranteed to be in the range [0.0, 1.0].
    #[inline]
    pub fn as_f32(self) -> f32 {
        self.0
    }

    /// Performs linear interpolation between this value and another.
    #[inline]
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        let t = t.clamp(Self::MIN, Self::MAX);
        Self::new(self.0 * (1.0 - t) + other.0 * t)
    }

    /// Converts the value to a 1-byte representation ([`u8`]).
    #[inline]
    pub fn to_u8(&self) -> u8 {
        (self.0 * 255.0).round().clamp(0.0, 255.0) as u8
    }

    /// Converts the value to a 2-byte representation (`[u8; 2]`), big-endian.
    #[inline]
    pub fn to_u16_bytes(&self) -> [u8; 2] {
        let val = (self.0 * 65535.0).round().clamp(0.0, 65535.0) as u16;
        val.to_be_bytes()
    }

    /// Converts the value to a 3-byte representation (`[u8; 3]`), big-endian.
    #[inline]
    pub fn to_u24_bytes(&self) -> [u8; 3] {
        let val = (self.0 * 16777215.0).round().clamp(0.0, 16777215.0) as u32;
        [((val >> 16) & 0xFF) as u8, ((val >> 8) & 0xFF) as u8, (val & 0xFF) as u8]
    }

    /// Converts the value to a 4-byte representation (`[u8; 4]`), big-endian.
    #[inline]
    pub fn to_u32_bytes(&self) -> [u8; 4] {
        let val = (self.0 * 4294967295.0).round().clamp(0.0, 4294967295.0) as u32;
        val.to_be_bytes()
    }

    /// Converts the value to values directly mappable at addresses.
    pub fn to_address_values(&self, addresses: &[Address]) -> Vec<(Address, theymx::Value)> {
        let bytes: Vec<u8> = match addresses.len() {
            1 => vec![self.to_u8()],
            2 => self.to_u16_bytes().to_vec(),
            3 => self.to_u24_bytes().to_vec(),
            4 => self.to_u32_bytes().to_vec(),
            _ => {
                log::warn!(
                    "cannot set DMX channel value for fixture: unsupported address length {}",
                    addresses.len()
                );
                return Vec::new();
            }
        };

        addresses.iter().copied().zip(bytes.into_iter().map(theymx::Value::from)).collect()
    }
}

impl fmt::Display for ClampedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<f32> for ClampedValue {
    fn from(value: f32) -> Self {
        Self::new(value)
    }
}

impl From<ClampedValue> for f32 {
    fn from(value: ClampedValue) -> Self {
        value.0
    }
}

impl From<ClampedValue> for f64 {
    fn from(value: ClampedValue) -> Self {
        value.0 as f64
    }
}

impl From<ClampedValue> for theymx::Value {
    fn from(value: ClampedValue) -> Self {
        theymx::Value((value.0 * (u8::MAX as f32)) as u8)
    }
}

impl str::FromStr for ClampedValue {
    type Err = num::ParseFloatError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.parse()?))
    }
}

/// Stores clamped attribute values for each fixture id.
///
/// [`AttributeValues`] maintains a mapping from [`FixtureId`] to a set of
/// attribute-value pairs, where each value is a [`ClampedValue`] in the range [0.0, 1.0].
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct AttributeValues {
    values: HashMap<FixtureId, HashMap<Attribute, ClampedValue>>,
}

impl Default for AttributeValues {
    fn default() -> Self {
        Self::new()
    }
}

impl AttributeValues {
    /// Creates a new, empty [`AttributeValues`] collection.
    pub fn new() -> Self {
        Self { values: HashMap::new() }
    }

    /// Sets the value for a given attribute at a specific fixture path.
    ///
    /// If the fixture path or attribute does not exist, it will be created.
    /// The value is converted into a [`ClampedValue`] and stored.
    pub fn set(
        &mut self,
        fixture_id: FixtureId,
        attribute: Attribute,
        value: impl Into<ClampedValue>,
    ) {
        self.values.entry(fixture_id).or_default().insert(attribute, value.into());
    }

    /// Returns an iterator over all stored attribute values.
    pub fn values(&self) -> impl Iterator<Item = (&FixtureId, &Attribute, &ClampedValue)> {
        // Annotate the closure parameter types so the compiler can infer everything
        // inside the nested iterator correctly.
        self.values.iter().flat_map(
            |(fixture_id, attrs): (&FixtureId, &HashMap<Attribute, ClampedValue>)| {
                attrs
                    .iter()
                    .map(move |(attr, val): (&Attribute, &ClampedValue)| (fixture_id, attr, val))
            },
        )
    }

    /// Retrieves the value for a given attribute at a specific fixture path, if present.
    pub fn get(&self, id: &FixtureId, attribute: &Attribute) -> Option<ClampedValue> {
        self.values
            .get(id)
            .and_then(|attrs: &HashMap<Attribute, ClampedValue>| attrs.get(attribute))
            .copied()
    }
}
