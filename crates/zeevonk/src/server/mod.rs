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
//! For more advanced usage, see the documentation for [`Server`].

use std::sync::Arc;

use crate::project::Project;
use crate::project::definition::ProjectDefinition;
use crate::server::client::controller::{ControllerListener, ControllerManager};
use crate::server::client::processor::{ProcessorListener, ProcessorManager};
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
    controller_agent: Arc<ControllerManager>,
    processor_agent: Arc<ProcessorManager>,
    router: Arc<Router>,
}

impl Server {
    /// Creates a new [`Server`] instance.
    pub fn new(project: ProjectDefinition) -> crate::Result<Self> {
        let project_handle = Arc::new(project_builder::from_definition(project)?);

        let output_agent = Arc::new(OutputAgent::new(project_handle.clone()));
        let controller_agent = Arc::new(ControllerManager::new());
        let processor_agent = Arc::new(ProcessorManager::new());

        let router = Arc::new(Router::new(
            project_handle.clone(),
            controller_agent.clone(),
            processor_agent.clone(),
        ));

        Ok(Self {
            project: Arc::clone(&project_handle),
            output_agent,
            controller_agent,
            processor_agent,
            router,
        })
    }

    /// Starts the server instance and its listeners.
    pub async fn start(&self) -> crate::server::Result<()> {
        self.output_agent.start();

        let project = &self.project;
        let controller_port = project.config_definition().controller_port;
        let processor_port = project.config_definition().processor_port;

        let (controller_res, processor_res) = tokio::join!(
            ControllerListener::start(
                self.controller_agent.clone(),
                self.router.clone(),
                controller_port
            ),
            ProcessorListener::start(
                self.processor_agent.clone(),
                self.output_agent.clone(),
                processor_port
            )
        );

        controller_res?;
        processor_res?;

        Ok(())
    }

    /// Returns a reference to the [`Project`].
    pub fn project(&self) -> &Project {
        &self.project
    }
}
