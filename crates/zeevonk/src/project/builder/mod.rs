use crate::project::{Project, file::ProjectFile};

mod stage;

/// Converts a [`ProjectFile`] to a [`Project`].
pub fn from_file(file: ProjectFile) -> crate::Result<Project> {
    let stage = stage::from_file(&file)?;

    Ok(Project { file, stage })
}
