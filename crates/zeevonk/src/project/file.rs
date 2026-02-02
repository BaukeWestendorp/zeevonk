//! Project file management module.

use std::path::Path;
use std::{fs, io};

/// The relative path to the project description file within a project folder.
const RELATIVE_DESCRIPTION_FILE_PATH: &str = "project.json";
/// The relative path to the directory containing GDTF files within a project folder.
const RELATIVE_GDTF_FILES_PATH: &str = "gdtf_files";

/// Represents the main project file, containing all configuration sections.
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ProjectFile {
    /// The patch configuration, including fixtures and GDTF file references.
    pub patch: patch::Patch,
    /// The DMX output configuration for the project.
    pub dmx_output: dmx_output::DmxOutputDefinition,
}

impl ProjectFile {
    /// Loads a [`ProjectFile`] from the specified project folder.
    ///
    /// # Errors
    ///
    /// Returns an error if the project file cannot be read, deserialized
    /// or if the GDTF files directory cannot be read.
    pub fn load_from_folder(project_path: &Path) -> crate::Result<Self> {
        // Load project from description file.
        let file = fs::File::open(project_path.join(RELATIVE_DESCRIPTION_FILE_PATH))?;
        let mut project: ProjectFile = serde_json::from_reader(file).map_err(|err| {
            io::Error::other(format!("failed to deserialize project file: {err}"))
        })?;

        // Get GDTF file paths.
        let gdtf_dir_path = project_path.join(RELATIVE_GDTF_FILES_PATH);
        let gdtf_file_dir = fs::read_dir(&gdtf_dir_path)?;
        for entry in gdtf_file_dir {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let file_path = entry.path();

            if file_path.extension().and_then(|ext| ext.to_str()) != Some("gdtf") {
                continue;
            }

            project.patch.gdtf_file_paths.push(file_path);
        }

        Ok(project)
    }

    /// Saves the [`ProjectFile`] to the specified project folder.
    ///
    /// # Errors
    ///
    /// Returns an error if the description file cannot be written or if any GDTF file cannot be copied.
    pub fn save_to_folder(&self, project_path: &Path) -> Result<(), crate::Error> {
        // Ensure the gdtf_files directory exists.
        let gdtf_dir = project_path.join(RELATIVE_GDTF_FILES_PATH);
        fs::create_dir_all(&gdtf_dir)?;

        // Save the project description.
        let description_path = project_path.join(RELATIVE_DESCRIPTION_FILE_PATH);
        let project_to_save = self.clone();

        let file = fs::File::create(&description_path)?;
        serde_json::to_writer_pretty(file, &project_to_save)
            .map_err(|err| io::Error::other(format!("failed to write project file: {err}")))?;

        // Copy GDTF files into the gdtf_files directory.
        for path in &self.patch.gdtf_file_paths {
            if let Some(filename) = path.file_name() {
                let dest = gdtf_dir.join(filename);
                // Only copy if source and destination are different.
                if path != &dest {
                    fs::copy(path, &dest)?;
                }
            }
        }

        Ok(())
    }
}

pub mod patch {
    //! Contains types and files related to patching fixtures.

    use std::path::PathBuf;

    use theymx::Address;
    use uuid::Uuid;

    use crate::project::stage::FixtureIdPart;

    /// Defines the stage configuration.
    #[derive(Debug, Clone, PartialEq)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct Patch {
        /// The list of GDTF file paths used in this stage.
        #[serde(skip)]
        pub gdtf_file_paths: Vec<PathBuf>,

        /// The list of fixture files in the stage.
        pub fixtures: Vec<FixtureDefinition>,
    }

    /// Represents a single fixture instance in the stage.
    #[derive(Debug, Clone, PartialEq)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct FixtureDefinition {
        /// The name of the fixture.
        pub name: String,
        /// The root fixture id for the fixture.
        pub root_id: FixtureIdPart,
        /// The DMX address for the fixture.
        pub address: Address,
        /// The kind of fixture, including GDTF type and DMX mode.
        pub kind: FixtureKindDefinition,
    }

    /// Describes the type and DMX mode of a fixture, referencing a GDTF fixture type.
    #[derive(Debug, Clone, PartialEq)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct FixtureKindDefinition {
        /// The UUID of the GDTF fixture type.
        pub gdtf_fixture_type_id: Uuid,
        /// The name of the DMX mode for this fixture.
        pub gdtf_dmx_mode: String,
    }
}

pub mod dmx_output {
    //! Contains types and definitions related to DMX output configuration.

    use std::net::SocketAddr;

    use theymx::UniverseId;

    /// Defines the DMX output configuration for a project.
    #[derive(Debug, Clone, PartialEq)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct DmxOutputDefinition {
        /// The list of DMX output instances.
        pub instances: Vec<DmxOutputInstanceDefinition>,
    }

    /// Represents a single DMX output instance, such as a hardware interface.
    #[derive(Debug, Clone, PartialEq)]
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum DmxOutputInstanceDefinition {
        /// An Enttec Open DMX USB interface.
        EnttecOpenDmx {
            /// The universe ID this output instance is assigned to.
            universe_id: UniverseId,
            /// The serial number of the Enttec Open DMX device.
            serial_number: String,
        },
        /// A sACN network output.
        Sacn {
            /// The name of this sACN output instance.
            name: String,
            /// The universe IDs this output instance is assigned to.
            universe_ids: Vec<UniverseId>,
            /// Whether this output is in preview mode.
            preview_mode: bool,
            /// The sACN priority for this output.
            priority: u8,
            /// The address to send the sACN output to.
            target_address: SocketAddr,
        },
    }
}
