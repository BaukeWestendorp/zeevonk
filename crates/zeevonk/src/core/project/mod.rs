//! This module contains the definitions for the project file, and the resolved values
//! for the project.

pub mod definition;
pub mod dmx_output;
pub mod patch;

pub(crate) mod builder;

use crate::project::definition::dmx_output::DmxOutputDefinition;
use crate::project::definition::patch::PatchDefinition;
use crate::project::dmx_output::DmxOutput;
use crate::project::patch::Patch;

/// Represents a complete project, including patch and DMX output configuration.
pub struct Project {
    patch: Patch,
    patch_definition: PatchDefinition,

    dmx_output: DmxOutput,
    dmx_output_definition: DmxOutputDefinition,
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
}
