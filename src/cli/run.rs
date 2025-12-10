use std::path::PathBuf;

use zeevonk::engine::Engine;
use zeevonk::showfile::Showfile;

/// Runs the showfile at the given path.
pub fn run_showfile(showfile_path: PathBuf) -> anyhow::Result<()> {
    let showfile = Showfile::load_from_folder(&showfile_path)?;

    let mut engine = Engine::new(&showfile);
    engine.start()?;

    Ok(())
}
