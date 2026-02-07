#![warn(missing_docs)]

//! # Zeevonk
//!
//! A modular lighting control system for modern DMX-based lighting setups.
//!
//! > ⚠️ **Warning**
//! >
//! > Zeevonk is currently in early development. APIs, features, and behavior may change frequently and without notice.
//! > It is **not yet recommended for production use**.
//!
//! ## What is Zeevonk?
//!
//! Zeevonk is a modular system for controlling lighting fixtures.
//!
//! This project is the result of a deep rabbithole I went into, when creating [Radiant](https://github.com/BaukeWestendorp/radiant). I realized I was writing the same DMX resolvers for GDTF files over and over again. Zeevonk is my way of consolidating all of my research into a hub for DMX lighting.
//!
//! For more details, see the documentation for each module in the crate.

pub mod attr;
pub mod error;
pub mod project;
pub mod value;

mod output;
mod project_builder;
mod resolver;

/// Re-export of the [`theymx`](https://github.com/BaukeWestendorp/theymx) crate.
pub use theymx;

use std::sync::Arc;

use crate::output::agent::OutputAgent;
use crate::project::Project;
use crate::project::file::ProjectFile;
use crate::value::AttributeValues;

pub use error::{Error, Result};

/// The main entry point for interacting with a Zeevonk instance.
pub struct Zeevonk {
    project: Arc<Project>,
    output_agent: Arc<OutputAgent>,
}

impl Zeevonk {
    /// Creates a new [`Zeevonk`] instance from a [`ProjectFile`].
    ///
    /// # Errors
    ///
    /// Returns an error if the project file cannot be loaded or parsed.
    pub fn new(project_file: ProjectFile) -> crate::Result<Self> {
        let project_handle = Arc::new(project_builder::from_file(project_file)?);
        let output_agent = Arc::new(OutputAgent::new(project_handle.clone()));
        Ok(Self { project: Arc::clone(&project_handle), output_agent })
    }

    /// Starts Zeevonk.
    pub fn start(&self) {
        self.output_agent.start();
        log::info!("Zeevonk started");
    }

    /// Returns a reference to the loaded [`Project`].
    pub fn project(&self) -> &Project {
        &self.project
    }

    /// Sets attribute values for fixtures in the project.
    ///
    /// This method updates the output agent with the provided values. If `include_children` is
    /// `true`, the values are also applied recursively to all child fixtures.
    pub fn set_attribute_values(&self, values: AttributeValues) {
        log::debug!("setting attribute values");
        self.output_agent.update_values(values.clone());
    }
}
