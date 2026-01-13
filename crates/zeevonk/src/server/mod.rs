//! The Zeevonk server serves as a hub to connect multiple clients
//! together and generating DMX output over various protocols.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::RwLockReadGuard;

use crate::server::showfile::Showfile;
use crate::server::state::ServerState;
use crate::show::ShowData;

pub mod showfile;

mod controller;
mod error;
mod output;
mod resolver;
mod state;

pub use error::Error;

pub struct Server<'sf> {
    showfile: &'sf Showfile,
    state: Arc<ServerState>,

    bound_addr: Option<SocketAddr>,
}

impl<'sf> Server<'sf> {
    pub fn new(showfile: &'sf Showfile) -> Result<Self, Error> {
        let init_start = Instant::now();

        let state = Arc::new(ServerState::new(showfile)?);

        let init_duration = init_start.elapsed();
        log::info!("zeevonk server initialized (init time: {:.2?})", init_duration);

        Ok(Self { showfile, state, bound_addr: None })
    }

    pub async fn start(&mut self) -> Result<(), Error> {
        log::info!("starting server...");
        let startup_start = Instant::now();

        // Initialize state.
        let state = Arc::clone(&self.state);

        // Start protocol manager.
        log::debug!("starting protocol manager");
        output::agent::start(self.showfile.protocols().clone(), Arc::clone(&state));
        log::debug!("protocol manager started");

        let startup_duration = startup_start.elapsed();
        log::info!("server startup complete (startup time: {:.2?})", startup_duration);

        // Start controller listener.
        let controller_port = self.showfile.config().controller_port();
        let controller_addr =
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, controller_port));
        controller::start_listener(controller_addr).await?;

        Ok(())
    }

    /// Returns the address the socket has been bound to.
    ///
    /// # Panics
    ///
    /// Panics if the server has not been started yet.
    pub fn address(&self) -> SocketAddr {
        self.bound_addr.expect("server should have been started before calling this")
    }

    pub fn show_data(&'_ self) -> RwLockReadGuard<'_, ShowData> {
        self.state.show_data.blocking_read()
    }
}
