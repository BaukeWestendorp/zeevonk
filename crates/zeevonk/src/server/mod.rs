//! The Zeevonk server serves as a hub to connect multiple clients
//! together and generating DMX output over various protocols.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use theymx::Multiverse;
use tokio::sync::{RwLock, RwLockReadGuard};
use warp::Filter;

use crate::attr::Attribute;
use crate::show::ShowData;
use crate::show::fixture::FixturePath;
use crate::showfile::Showfile;
use crate::value::{AttributeValues, ClampedValue};

mod error;
mod protocols;
mod resolver;
mod show_data_builder;

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
        protocols::agent::start(self.showfile.protocols().clone(), Arc::clone(&state));
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

        let routes = get_show_data
            .or(get_dmx_output)
            .or(post_attribute_values)
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

#[derive(Debug)]
struct ServerState {
    show_data: RwLock<ShowData>,

    pending_attribute_values: RwLock<AttributeValues>,
    output_multiverse: RwLock<Multiverse>,
}

impl ServerState {
    pub fn new<'sf>(showfile: &'sf Showfile) -> Result<Self, Error> {
        let show_data = show_data_builder::build_from_showfile(showfile)?;

        Ok(Self {
            show_data: RwLock::new(show_data),

            pending_attribute_values: RwLock::new(AttributeValues::new()),
            output_multiverse: RwLock::new(Multiverse::new()),
        })
    }

    async fn set_attribute_value(
        &self,
        fixture_path: FixturePath,
        attribute: Attribute,
        value: ClampedValue,
    ) {
        self.pending_attribute_values.write().await.set(fixture_path, attribute, value);
    }
}
