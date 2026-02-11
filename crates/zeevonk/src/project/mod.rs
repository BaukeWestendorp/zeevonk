//! Project file and stage information.

pub mod file;
pub mod stage;

use crate::project::{file::ProjectFile, stage::Stage};

#[doc(hidden)]
pub mod builder;

/// Represents a project, containing its file and stage information.
#[derive(Debug, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Project {
    pub(crate) file: ProjectFile,

    pub(crate) stage: Stage,
}

impl Project {
    /// Returns a reference to the project's file.
    pub fn file(&self) -> &ProjectFile {
        &self.file
    }

    /// Returns a reference to the project's stage.
    pub fn stage(&self) -> &Stage {
        &self.stage
    }
}
