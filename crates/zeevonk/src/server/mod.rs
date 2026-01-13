//! The Zeevonk server serves as a hub to connect multiple clients
//! together and generating DMX output over various protocols.

use std::sync::Arc;
use std::time::Instant;

use tokio::sync::RwLockReadGuard;

use crate::server::showfile::Showfile;
use crate::server::state::State;
use crate::show::ShowData;

pub mod showfile;

mod controller;
mod error;
mod output;
mod state;

pub use error::Error;

pub struct Server<'sf> {
    showfile: &'sf Showfile,
    state: Arc<State>,
}

impl<'sf> Server<'sf> {
    pub fn new(showfile: &'sf Showfile) -> Result<Self, Error> {
        let init_start = Instant::now();

        let state = Arc::new(State::new(showfile)?);

        let init_duration = init_start.elapsed();
        log::info!("zeevonk server initialized (init time: {:.2?})", init_duration);

        Ok(Self { showfile, state })
    }

    pub async fn start(&self) -> Result<(), Error> {
        log::info!("starting server...");
        let startup_start = Instant::now();

        // Start protocol manager.
        log::debug!("starting protocol manager");
        let state = Arc::clone(&self.state);
        output::agent::start(self.showfile.protocols().clone(), state);
        log::debug!("protocol manager started");

        let startup_duration = startup_start.elapsed();
        log::info!("server startup complete (startup time: {:.2?})", startup_duration);

        Ok(())
    }

    pub fn show_data(&self) -> RwLockReadGuard<'_, ShowData> {
        self.state.show_data.blocking_read()
    }
}
