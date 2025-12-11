use std::path::PathBuf;

use zeevonk::server::Server;
use zeevonk::core::showfile::Showfile;

/// Runs the showfile at the given path.
pub fn run_showfile(showfile_path: PathBuf) -> anyhow::Result<()> {
    let showfile = Showfile::load_from_folder(&showfile_path)?;

    let mut server = Server::new(&showfile);
    server.start()?;

    Ok(())
}
