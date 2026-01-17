#![warn(missing_docs)]

//! # Zeevonk Server
//!
//! The Zeevonk server implementation, which serves as a hub
//! for Zeevonk Clients (e.g. controllers and processors). It also has built in
//! support for resolving attribute values into DMX universes and sending them
//! over various output protocols like sACN.

use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;
use std::thread;

use crate::project::Project;
use crate::project::definition::ProjectDefinition;
use crate::server::output::agent::OutputAgent;
use crate::server::processor::ProcessorListener;

pub mod error;
mod output;
mod processor;
mod resolver;

mod project_builder;

pub use error::{Error, Result};

/// The main interface to start and manage a Zeevonk server.
pub struct Server {
    project: Arc<Project>,

    output_agent: Arc<OutputAgent>,
}

impl Server {
    /// Creates a new [`Server`] instance.
    pub fn new(project: ProjectDefinition) -> crate::Result<Self> {
        let project_handle = Arc::new(project_builder::from_definition(project)?);

        Ok(Self {
            project: Arc::clone(&project_handle),
            output_agent: Arc::new(OutputAgent::new(project_handle)),
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
    }

    /// Returns a reference to the [`Project`] associated with this server.
    pub fn project(&self) -> &Project {
        &self.project
    }
}
