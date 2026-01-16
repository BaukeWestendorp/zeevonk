use theymx::Multiverse;

/// Represents a DMX output, holding a default multiverse.
pub struct DmxOutput {
    pub(crate) default_multiverse: Multiverse,
}

impl DmxOutput {
    /// Returns a reference to the default [`Multiverse`] used by this output.
    pub fn default_multiverse(&self) -> &Multiverse {
        &self.default_multiverse
    }
}
