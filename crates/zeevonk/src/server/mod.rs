#![warn(missing_docs)]

//! # Zeevonk Server
//!
//! The Zeevonk server implementation, which serves as a hub
//! for Zeevonk Clients (e.g. controllers and processors). It also has built in
//! support for resolving attribute values into DMX universes and sending them
//! over various output protocols like sACN.

use std::sync::Arc;

use crate::project::definition::ProjectDefinition;
use crate::project::{self, Project};
use crate::server::output::agent::OutputAgent;

pub mod error;
mod output;
mod resolver;

pub use error::{Error, Result};

/// The main interface to start and manage a Zeevonk server.
pub struct Server {
    project: Arc<Project>,

    output_agent: OutputAgent,
}

impl Server {
    /// Creates a new [`Server`] instance.
    pub fn new(project: ProjectDefinition) -> crate::Result<Self> {
        let project_handle = Arc::new(project::builder::from_definition(project)?);

        Ok(Self {
            project: Arc::clone(&project_handle),
            output_agent: OutputAgent::new(project_handle),
        })
    }

    /// Starts the server instance and its listeners.
    pub fn start(&self) {
        self.output_agent().start();
    }

    // FIXME: REMOVE
    pub fn test_send(&self, values: crate::value::AttributeValues) {
        self.output_agent().test_send(values);
    }

    /// Returns a reference to the [`Project`] associated with this server.
    pub fn project(&self) -> &Project {
        &self.project
    }

    pub(crate) fn output_agent(&self) -> &OutputAgent {
        &self.output_agent
    }
}
