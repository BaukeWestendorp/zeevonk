use std::path::PathBuf;

use zeevonk::project::definition::ProjectDefinition;
use zeevonk::server::Server;

/// Runs the project at the given path.
pub fn run_project(project_path: PathBuf) -> anyhow::Result<()> {
    let project_definition = ProjectDefinition::load_from_folder(&project_path)?;
    let server = Server::new(project_definition)?;
    server.start();

    loop {
        std::thread::sleep(std::time::Duration::from_secs_f32(1.0 / 60.0));
    }
}
