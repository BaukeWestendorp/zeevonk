//! FIXME: Refactor and then improve docs.

pub mod definition;
pub mod dmx_output;
pub mod patch;

use crate::project::definition::config::ConfigDefinition;
use crate::project::definition::dmx_output::DmxOutputDefinition;
use crate::project::definition::patch::PatchDefinition;
use crate::project::definition::router::RouterDefinition;
use crate::project::dmx_output::DmxOutput;
use crate::project::patch::Patch;

/// Represents a complete project.
pub struct Project {
    pub(crate) patch: Patch,
    pub(crate) patch_definition: PatchDefinition,

    pub(crate) dmx_output: DmxOutput,
    pub(crate) dmx_output_definition: DmxOutputDefinition,

    pub(crate) config_definition: ConfigDefinition,

    pub(crate) router_definition: RouterDefinition,
}

impl Project {
    /// Returns a reference to the patch.
    pub fn patch(&self) -> &Patch {
        &self.patch
    }

    /// Returns a reference to the patch definition.
    pub fn patch_definition(&self) -> &PatchDefinition {
        &self.patch_definition
    }

    /// Returns a reference to the DMX output.
    pub fn dmx_output(&self) -> &DmxOutput {
        &self.dmx_output
    }

    /// Returns a reference to the DMX output definition.
    pub fn dmx_output_definition(&self) -> &DmxOutputDefinition {
        &self.dmx_output_definition
    }

    /// Returns a reference to the config definition.
    pub fn config_definition(&self) -> &ConfigDefinition {
        &self.config_definition
    }

    /// Returns a reference to the router definition.
    pub fn router_definition(&self) -> &RouterDefinition {
        &self.router_definition
    }
}
