//! The Zeevonk server manages clients and outputs DMX.
//!
//! This server acts as the central hub for your lighting control system.
//! It receives triggers and attribute updates from processor clients and outputs DMX data using various protocols.
//!
//! ## Example
//!
//! ```no_run
//! use std::path::Path;
//!
//! use zeevonk::server::Server;
//! use zeevonk::project::file::ProjectFile;
//!
//! // Create a project definition.
//! let project_def = ProjectFile::load_from_folder(&Path::new("path/to/project_folder")).unwrap();
//!
//! // Create and start the server.
//! let server = Server::new(project_def).unwrap();
//! server.start();
//! ```
//!
//! The server will now listen for clients, and handle DMX output automatically.

use std::sync::Arc;

use crate::project::Project;
use crate::project::file::ProjectFile;
use crate::server::client::{ClientAgent, ClientListener};
use crate::server::output::agent::OutputAgent;
use crate::server::router::Router;

pub mod error;

mod client;
mod output;
mod project_builder;
mod resolver;
mod router;

pub use error::{Error, Result};

/// The main interface to start and manage a Zeevonk server.
pub struct Server {
    project: Arc<Project>,
    output_agent: Arc<OutputAgent>,
    client_agent: Arc<ClientAgent>,
    router: Arc<Router>,
}

impl Server {
    /// Creates a new [`Server`] instance.
    pub fn new(project: ProjectFile) -> crate::Result<Self> {
        let project_handle = Arc::new(project_builder::from_file(project)?);

        let output_agent = Arc::new(OutputAgent::new(project_handle.clone()));
        let client_agent = Arc::new(ClientAgent::new());

        let router = Arc::new(Router::new(project_handle.clone(), client_agent.clone()));

        Ok(Self { project: Arc::clone(&project_handle), output_agent, client_agent, router })
    }

    /// Starts the server instance and its listeners.
    pub async fn start(&self) -> crate::server::Result<()> {
        self.output_agent.start();

        let port = self.project.file().config.port;

        ClientListener::start(
            self.client_agent.clone(),
            self.output_agent.clone(),
            self.router.clone(),
            self.project.clone(),
            port,
        )
        .await?;

        Ok(())
    }

    /// Returns a reference to the [`Project`].
    pub fn project(&self) -> &Project {
        &self.project
    }
}
