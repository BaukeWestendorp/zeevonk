use std::path::PathBuf;

use anyhow::Ok;
use zeevonk::server::Server;
use zeevonk::showfile::Showfile;

/// Runs the showfile at the given path.
pub fn run_showfile(showfile_path: PathBuf) -> anyhow::Result<()> {
    tokio::runtime::Builder::new_multi_thread().enable_io().build().unwrap().block_on(async {
        let showfile = Showfile::load_from_folder(&showfile_path)?;
        let mut server = Server::new(&showfile)?;
        server.start().await?;

        anyhow::Result::<()>::Ok(())
    })?;

    Ok(())
}
