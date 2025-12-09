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
