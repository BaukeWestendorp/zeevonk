use std::fs;

use anyhow::Context;

use zeevonk::showfile::Showfile;

fn main() -> anyhow::Result<()> {
    let showfile_path = "example_config.json";
    let file = fs::File::open(showfile_path).context("failed to open showfile")?;
    let showfile: Showfile =
        serde_json::from_reader(file).context("failed to deserialize showfile")?;

    dbg!(showfile);

    Ok(())
}
