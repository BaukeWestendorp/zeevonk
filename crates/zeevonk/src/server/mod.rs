//! The Zeevonk server manages clients and the data they send, and sends DMX output to various protocols.
//!
//! This server acts as the central hub for your lighting control system.
//! It receives triggers from controller clients, processes attribute
//! updates from processor clients, and outputs DMX data using various protocols.
//!
//! ## Example
//!
//! ```no_run
//! use std::path::Path;
//!
//! use zeevonk::server::Server;
//! use zeevonk::project::definition::ProjectDefinition;
//!
//! // Create a project definition.
//! let project_def = ProjectDefinition::load_from_folder(&Path::new("path/to/project_folder")).unwrap();
//!
//! // Create and start the server.
//! let server = Server::new(project_def).unwrap();
//! server.start();
//! ```
//!
//! The server will now listen for processor and controller clients, and handle DMX output automatically.
//!
//! For more advanced usage, see the documentation for [`Server`](crate::server::Server).

use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;
use std::thread;

use crate::project::Project;
use crate::project::definition::ProjectDefinition;
use crate::server::controller::ControllerListener;
use crate::server::output::agent::OutputAgent;
use crate::server::processor::ProcessorListener;
use crate::server::router::Router;

pub mod error;

mod controller;
mod output;
mod processor;
mod project_builder;
mod resolver;
mod router;

pub use error::{Error, Result};

/// The main interface to start and manage a Zeevonk server.
pub struct Server {
    project: Arc<Project>,

    output_agent: Arc<OutputAgent>,
    router: Arc<Router>,
}

impl Server {
    /// Creates a new [`Server`] instance.
    pub fn new(project: ProjectDefinition) -> crate::Result<Self> {
        let project_handle = Arc::new(project_builder::from_definition(project)?);

        Ok(Self {
            project: Arc::clone(&project_handle),

            output_agent: Arc::new(OutputAgent::new(project_handle.clone())),
            router: Arc::new(Router::new(project_handle)),
        })
    }

    /// Starts the server instance and its listeners.
    pub fn start(&self) {
        self.output_agent.start();

        thread::spawn({
            let port = self.project.config_definition().processor_port;
            let address = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
            let output_agent = Arc::clone(&self.output_agent);
            move || ProcessorListener::new(output_agent).start(address)
        });

        thread::spawn({
            let port = self.project.config_definition().controller_port;
            let address = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
            let router = Arc::clone(&self.router);
            move || ControllerListener::new(router).start(address)
        });
    }

    /// Returns a reference to the [`Project`]..
    pub fn project(&self) -> &Project {
        &self.project
    }

    /// Returns a reference to the [`Router`]..
    pub fn router(&self) -> &Router {
        &self.router
    }
}
