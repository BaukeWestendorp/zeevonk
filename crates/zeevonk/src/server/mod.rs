//! A hub for managing clients, resolving attributes and sending DMX.
//!
//! **Note:** The `server` feature must be enabled to start and manage a server from your
//! own code. If you prefer a ready-made program instead of embedding a server, use the standalone
//! zeevonk command-line tool. See the [`zeevonk` CLI](FIXME) for installation and usage details.
//!
//! The Zeevonk server is a hub for managing clients. It has a few essential responsibilities:
//! - Receiving [triggers](crate::trigger) from [controller clients](crate::client::controller) and routing them
//!   to the correct [processor clients](crate::client::processor).
//! - Receiving attribute updates from [processor clients](crate::client::processor)
//!   and converting them to DMX output.
//! - Sending DMX output over [various protocols](crate::project::dmx_output)
//!   like [sACN](crate::project::definition::dmx_output::DmxOutputInstanceDefinition)
//!   or [Entecc Open DMX](crate::project::definition::dmx_output::DmxOutputInstanceDefinition).
//!
//! # Examples
//!
//! FIXME: Add examples.

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
