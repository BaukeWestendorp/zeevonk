#![warn(missing_docs)]

//! Zeevonk.

pub mod attr;
pub mod error;
pub mod project;
pub mod value;

mod output;
mod project_builder;
mod resolver;

use std::sync::Arc;

use crate::attr::Attribute;
use crate::output::agent::OutputAgent;
use crate::project::Project;
use crate::project::file::ProjectFile;
use crate::project::stage::FixtureId;
use crate::value::{AttributeValues, ClampedValue};

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
    pub fn set_attribute_values(&self, values: AttributeValues, include_children: bool) {
        log::debug!("setting attribute values | include_children={}", include_children);
        self.output_agent.update_values(values.clone());

        if include_children {
            /// Recursively sets attribute values for child fixtures.
            fn set_values_recursively(
                project: &Project,
                output_agent: &OutputAgent,
                fixture_id: &FixtureId,
                attribute: Attribute,
                value: ClampedValue,
            ) {
                let Some(fixture) = project.stage().fixtures().get(fixture_id) else {
                    return;
                };

                for sub_id in fixture.sub_ids() {
                    output_agent.update_value(*sub_id, attribute, value);
                    set_values_recursively(project, output_agent, &sub_id, attribute, value);
                }
            }

            for (fixture_id, attribute, value) in values.values() {
                set_values_recursively(
                    &self.project,
                    &self.output_agent,
                    fixture_id,
                    *attribute,
                    *value,
                );
            }
        }
    }
}
