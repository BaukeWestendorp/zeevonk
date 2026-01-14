#![warn(missing_docs)]

//! # Zeevonk Server
//!
//! This crate contains the Zeevonk server implementation, which serves as a hub
//! for Zeevonk Clients (e.g. controllers and processors). It also has built in
//! support for resolving attribute values into DMX universes and sending them
//! over various output protocols like sACN.

use theymx::Multiverse;

use crate::output::agent::OutputAgent;

mod output;

/// The main interface to start and manage a Zeevonk server.
pub struct Server {
    output_agent: OutputAgent,
}

impl Server {
    /// Creates a new [`Server`] instance.
    pub fn new() -> Self {
        Self { output_agent: OutputAgent::new() }
    }

    pub(crate) fn output_agent(&self) -> &OutputAgent {
        &self.output_agent
    }

    /// Returns the latest [`Multiverse`] containing the resolved DMX
    /// data of all universes that are used by this server.
    pub fn multiverse(&self) -> &Multiverse {
        self.output_agent().multiverse()
    }
}
