//! Identifiers for tracking triggers or clients.

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
    ///
    /// ```rust
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// use zeevonk::ident::Identifier;
    /// let id = Identifier::new("client-01")?;
    /// assert_eq!(id.as_str(), "client-01");
    /// # Ok(()) }
    /// ```
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

    /// Consume the identifier and return the owned String.
    pub fn into_inner(self) -> String {
        self.0
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

impl AsRef<str> for Identifier {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Identifier {
    type Error = crate::Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Identifier::new(s)
    }
}
impl<'a> TryFrom<&'a str> for Identifier {
    type Error = crate::Error;
    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        Identifier::new(s.to_owned())
    }
}
