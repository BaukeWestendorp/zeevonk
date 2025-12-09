use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use zeevonk::engine::Engine;
use zeevonk::showfile::Showfile;

/// Runs the showfile at the given path.
pub fn run_showfile(showfile_path: PathBuf) -> anyhow::Result<()> {
    let file = fs::File::open(showfile_path).context("failed to open showfile")?;
    let showfile: Showfile =
        serde_json::from_reader(file).context("failed to deserialize showfile")?;

    let mut engine = Engine::new(showfile);
    engine.start()?;

    Ok(())
}
