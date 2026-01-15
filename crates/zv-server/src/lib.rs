#![warn(missing_docs)]

//! # Zeevonk Server
//!
//! This crate contains the Zeevonk server implementation, which serves as a hub
//! for Zeevonk Clients (e.g. controllers and processors). It also has built in
//! support for resolving attribute values into DMX universes and sending them
//! over various output protocols like sACN.

use crate::output::agent::OutputAgent;

mod output;
mod project;

/// The main interface to start and manage a Zeevonk server.
pub struct Server {
    output_agent: OutputAgent,
}

impl Server {
    /// Creates a new [`Server`] instance.
    pub fn new() -> Self {
        Self { output_agent: OutputAgent::new() }
    }

    /// Starts the server instance and its listeners.
    pub fn start(&self) {
        self.output_agent().start();
    }

    // FIXME: REMOVE
    pub fn test_send(&self, values: std::collections::HashMap<u16, u8>) {
        self.output_agent().test_send(values);
    }

    pub(crate) fn output_agent(&self) -> &OutputAgent {
        &self.output_agent
    }
}
