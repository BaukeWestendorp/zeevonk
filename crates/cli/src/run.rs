use std::path::PathBuf;

use zeevonk::{Zeevonk, project::file::ProjectFile};

/// Runs the project at the given path.
pub fn run_project(project_path: PathBuf) -> anyhow::Result<()> {
    let project_definition = ProjectFile::load_from_folder(&project_path)?;
    let zeevonk = Zeevonk::new(project_definition)?;
    zeevonk.start();

    loop {
        std::thread::sleep(std::time::Duration::from_secs_f32(1.0 / 30.0));
    }
}
