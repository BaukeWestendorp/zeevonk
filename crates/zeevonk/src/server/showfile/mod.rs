use std::fs;
use std::path::{Path, PathBuf};

pub use config::*;
pub use error::*;
pub use patch::*;
pub use protocols::*;

mod config;
mod patch;
mod protocols;

mod error;

const RELATIVE_DESCRIPTION_FILE_PATH: &str = "showfile.json";
const RELATIVE_GDTF_FILES_PATH: &str = "gdtf_files";

// A showfile is the main configuration for Zeevonk.
#[derive(Debug, Clone, PartialEq, Default)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Showfile {
    #[serde(skip)]
    gdtf_file_paths: Vec<PathBuf>,

    config: Config,
    patch: Patch,
    protocols: Protocols,
}

impl Showfile {
    pub fn load_from_folder(showfile_path: &Path) -> Result<Self, Error> {
        // Load showfile from description file.
        let showfile_file = fs::File::open(showfile_path.join(RELATIVE_DESCRIPTION_FILE_PATH))?;
        let mut showfile: Showfile = serde_json::from_reader(showfile_file)
            .map_err(|e| Error::DeserializationError { message: e.to_string() })?;

        // Get GDTF file paths.
        let gdtf_dir_path = showfile_path.join(RELATIVE_GDTF_FILES_PATH);
        let gdtf_file_dir = fs::read_dir(&gdtf_dir_path)?;
        for entry in gdtf_file_dir {
            let Ok(entry) = entry else { continue };

            let file_path = entry.path();

            if !file_path.extension().is_some_and(|ext| ext == "gdtf") {
                continue;
            }

            showfile.gdtf_file_paths.push(file_path);
        }

        Ok(showfile)
    }

    pub fn save_to_folder(&self, showfile_path: &Path) -> Result<(), Error> {
        // Ensure the gdtf_files directory exists.
        let gdtf_dir = showfile_path.join(RELATIVE_GDTF_FILES_PATH);
        fs::create_dir_all(&gdtf_dir)?;

        // Save the showfile description.
        let description_path = showfile_path.join(RELATIVE_DESCRIPTION_FILE_PATH);
        let showfile_to_save = self.clone();

        let file = fs::File::create(&description_path)?;
        serde_json::to_writer_pretty(file, &showfile_to_save)
            .map_err(|e| Error::SerializationError { message: e.to_string() })?;

        // Copy GDTF files into the gdtf_files directory.
        for path in &self.gdtf_file_paths {
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

    pub fn gdtf_file_paths(&self) -> &[PathBuf] {
        &self.gdtf_file_paths
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn patch(&self) -> &Patch {
        &self.patch
    }

    pub fn protocols(&self) -> &Protocols {
        &self.protocols
    }
}
