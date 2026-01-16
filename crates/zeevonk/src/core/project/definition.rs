use std::path::Path;
use std::{fs, io};

const RELATIVE_DESCRIPTION_FILE_PATH: &str = "project.json";
const RELATIVE_GDTF_FILES_PATH: &str = "gdtf_files";

#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ProjectDefinition {
    pub patch: patch::PatchDefinition,
    pub dmx_output: dmx_output::DmxOutputDefinition,
}

impl ProjectDefinition {
    pub fn load_from_folder(project_path: &Path) -> crate::Result<Self> {
        // Load project from description file.
        let project_file = fs::File::open(project_path.join(RELATIVE_DESCRIPTION_FILE_PATH))?;
        let mut project: ProjectDefinition =
            serde_json::from_reader(project_file).map_err(|err| {
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
    use crate::project::patch::FixtureIdPart;
    use std::path::PathBuf;
    use theymx::Address;
    use uuid::Uuid;

    #[derive(Debug, Clone, PartialEq)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct PatchDefinition {
        #[serde(skip)]
        pub gdtf_file_paths: Vec<PathBuf>,

        pub fixtures: Vec<FixtureDefinition>,
    }

    #[derive(Debug, Clone, PartialEq)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct FixtureDefinition {
        pub name: String,
        pub root_id: FixtureIdPart,
        pub address: Address,
        pub kind: FixtureKindDefinition,
    }

    #[derive(Debug, Clone, PartialEq)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct FixtureKindDefinition {
        pub gdtf_fixture_type_id: Uuid,
        pub gdtf_dmx_mode: String,
    }
}

pub mod dmx_output {
    use theymx::UniverseId;

    #[derive(Debug, Clone, PartialEq)]
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct DmxOutputDefinition {
        pub instances: Vec<DmxOutputInstanceDefinition>,
    }

    #[derive(Debug, Clone, PartialEq)]
    #[derive(serde::Serialize, serde::Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum DmxOutputInstanceDefinition {
        EnttecOpenDmx { universe_id: UniverseId, serial_number: String },
    }
}
