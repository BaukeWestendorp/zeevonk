use theymx::Multiverse;

pub struct OutputAgent {
    /// The most recent multiverse that should be sent to all outputs.
    multiverse: Multiverse,
}

impl OutputAgent {
    pub fn new() -> Self {
        Self { multiverse: Multiverse::new() }
    }

    pub fn multiverse(&self) -> &Multiverse {
        &self.multiverse
    }
}
