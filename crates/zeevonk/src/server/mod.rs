//! The Zeevonk server serves as a hub to connect multiple clients
//! together and generating DMX output over various protocols.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::RwLockReadGuard;
use warp::Filter;

use crate::server::showfile::Showfile;
use crate::server::state::ServerState;
use crate::show::ShowData;
use crate::value::AttributeValues;

pub mod showfile;

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

        // Start server.
        let address = self.showfile.config().address();
        let get_show_data = warp::path("show-data").then({
            let state = Arc::clone(&state);
            move || {
                let state = Arc::clone(&state);
                async move {
                    let show_data = state.show_data.read().await.clone();
                    warp::reply::json(&show_data)
                }
            }
        });
        let get_dmx_output = warp::path("dmx-output").then({
            let state = Arc::clone(&state);
            move || {
                let state = Arc::clone(&state);
                async move {
                    state.resolve_values().await;
                    let multiverse = state.output_multiverse.read().await.clone();
                    warp::reply::json(&multiverse)
                }
            }
        });
        let post_attribute_values =
            warp::path("attribute-values").and(warp::body::json()).and(warp::post()).then({
                let state = Arc::clone(&state);
                move |values: AttributeValues| {
                    let state = Arc::clone(&state);
                    async move {
                        for (fixture_path, attribute, value) in values.values() {
                            state.set_attribute_value(*fixture_path, *attribute, *value).await;
                        }
                        state.resolve_values().await;
                        warp::reply::reply()
                    }
                }
            });
        let post_trigger = warp::path!("trigger" / String).and(warp::post()).then({
            let state = Arc::clone(&state);
            move |id: String| {
                let _state = Arc::clone(&state);
                async move {
                    // FIXME: Do something with trigger.
                    log::info!("received trigger: {}", id);
                    warp::reply::reply()
                }
            }
        });

        let routes = get_show_data
            .or(get_dmx_output)
            .or(post_attribute_values)
            .or(post_trigger)
            // FIXME: Figure out if this CORS is actually fine for our use case.
            .with(
                warp::cors()
                    .allow_any_origin()
                    .allow_headers(["Content-Type"])
                    .allow_methods(["GET", "POST", "PUT", "DELETE"]),
            );

        warp::serve(routes).run(address).await;

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
