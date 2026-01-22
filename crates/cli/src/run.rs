use std::path::PathBuf;

use zeevonk::{project::file::ProjectFile, server::Server};

/// Runs the project at the given path.
pub async fn run_project(project_path: PathBuf) -> anyhow::Result<()> {
    let project_definition = ProjectFile::load_from_folder(&project_path)?;
    let server = Server::new(project_definition)?;
    server.start().await?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs_f32(1.0 / 60.0));
    }
}
