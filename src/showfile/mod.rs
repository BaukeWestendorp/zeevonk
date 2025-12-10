use std::fmt;

use crate::showfile::config::Config;
use crate::showfile::patch::Patch;
use crate::showfile::protocols::Protocols;

/// General configuration.
pub mod config;
/// Patch definitions.
pub mod patch;
/// DMX IO protocols.
pub mod protocols;

/// The top-level showfile.
#[derive(Debug, Clone, PartialEq, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Showfile {
    config: Config,
    patch: Patch,
    protocols: Protocols,
}

impl Showfile {
    /// Returns a reference to the [`Config`] section of the showfile.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Returns a reference to the [`Patch`] section of the showfile.
    pub fn patch(&self) -> &Patch {
        &self.patch
    }

    /// Returns a reference to the [`Protocols`] section of the showfile.
    pub fn protocols(&self) -> &Protocols {
        &self.protocols
    }
}

/// Used for giving elements a visual label in the showfile.
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Label(String);

impl Label {
    /// Creates a new [`Label`].
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

/// A unique identifier, consisting of a namespace and a value.
/// The namespace represents a component. For Zeevonk, this will be 'zeevonk',
/// but for an external program this could be different.
/// The value is a unique thing within that namespace.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize)]
pub struct Identifier {
    /// The namespace represents a component. For Zeevonk, this will be 'zeevonk',
    /// but for an external program this could be different.
    namespace: String,
    /// The value is a unique thing within a namespace.
    value: String,
}

impl Identifier {
    /// Create a new Identifier, validating namespace and value.
    pub fn new(namespace: impl AsRef<str>, value: impl AsRef<str>) -> anyhow::Result<Self> {
        let mut id = Identifier::default();
        id.set_namespace(namespace.as_ref())?;
        id.set_value(value.as_ref())?;
        Ok(id)
    }

    /// Returns the namespace.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Returns the value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Sets the namespace if valid. Returns Ok(()) if successful, Err otherwise.
    pub fn set_namespace(&mut self, namespace: &str) -> anyhow::Result<()> {
        if Self::is_valid(namespace) {
            self.namespace = namespace.to_string();
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "invalid namespace: must be non-empty, lowercase, alphanumeric or '_'"
            ))
        }
    }

    /// Sets the value if valid. Returns Ok(()) if successful, Err otherwise.
    pub fn set_value(&mut self, value: &str) -> anyhow::Result<()> {
        if Self::is_valid(value) {
            self.value = value.to_string();
            Ok(())
        } else {
            Err(anyhow::anyhow!("invalid value: must be non-empty, lowercase, alphanumeric or '_'"))
        }
    }

    /// Validate that a string is lowercase, alphanumeric or '_'
    fn is_valid(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.value)
    }
}
