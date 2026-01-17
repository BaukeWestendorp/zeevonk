//! A validated identifier consisting of lowercase ASCII letters, digits, or hyphens.

/// A validated identifier consisting of lowercase ASCII letters, digits, or hyphens.
///
/// Use [`Identifier::new`] to construct a new identifier, which ensures the value
/// contains only valid characters. The underlying string can be accessed with [`Identifier::as_str`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Identifier(String);

impl Identifier {
    /// Creates a new [`Identifier`] after validating the input.
    ///
    /// Returns an error if the input contains invalid characters.
    pub fn new(id: impl Into<String>) -> crate::Result<Self> {
        let id_str = id.into();
        if id_str.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            Ok(Self(id_str))
        } else {
            Err(crate::Error::InvalidIdentifier)
        }
    }

    /// Returns a reference to the underlying string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for Identifier {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Identifier::new(s)
    }
}
