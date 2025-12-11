use std::path::{Path, PathBuf};
use std::{fmt, fs};

use anyhow::Context;

use crate::core::showfile::config::Config;
use crate::core::showfile::patch::Patch;
use crate::core::showfile::protocols::Protocols;

/// General configuration.
pub mod config;
/// Patch definitions.
pub mod patch;
/// DMX IO protocols.
pub mod protocols;

const RELATIVE_DESCRIPTION_FILE_PATH: &str = "showfile.json";
const RELATIVE_GDTF_FILES_PATH: &str = "gdtf_files";

/// The top-level showfile.
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
    /// Load a [`Showfile`] from a folder on disk.
    pub fn load_from_folder(showfile_path: &Path) -> anyhow::Result<Self> {
        // Load showfile from description file.
        let showfile_file = fs::File::open(showfile_path.join(RELATIVE_DESCRIPTION_FILE_PATH))
            .with_context(|| format!("failed to open '{}'", RELATIVE_DESCRIPTION_FILE_PATH))?;
        let mut showfile: Showfile = serde_json::from_reader(showfile_file)
            .with_context(|| format!("failed to parse '{}'", RELATIVE_DESCRIPTION_FILE_PATH))?;

        // Get GDTF file paths.
        let gdtf_file_dir = fs::read_dir(showfile_path.join(RELATIVE_GDTF_FILES_PATH))
            .context("failed to read gdtf_files directory")?;
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

    /// Save this [`Showfile`] into a folder on disk.
    pub fn save_to_folder(&self, showfile_path: &Path) -> anyhow::Result<()> {
        // Ensure the gdtf_files directory exists.
        let gdtf_dir = showfile_path.join(RELATIVE_GDTF_FILES_PATH);
        fs::create_dir_all(&gdtf_dir)
            .with_context(|| format!("failed to create '{}'", gdtf_dir.display()))?;

        // Save the showfile description.
        let description_path = showfile_path.join(RELATIVE_DESCRIPTION_FILE_PATH);
        let showfile_to_save = self.clone();

        let file = fs::File::create(&description_path)
            .with_context(|| format!("failed to create '{}'", description_path.display()))?;
        serde_json::to_writer_pretty(file, &showfile_to_save)
            .with_context(|| format!("failed to write '{}'", description_path.display()))?;

        // Copy GDTF files into the gdtf_files directory.
        for path in &self.gdtf_file_paths {
            if let Some(filename) = path.file_name() {
                let dest = gdtf_dir.join(filename);
                // Only copy if source and destination are different.
                if path != &dest {
                    fs::copy(path, &dest).with_context(|| {
                        format!("failed to copy '{}' to '{}'", path.display(), dest.display())
                    })?;
                }
            }
        }

        Ok(())
    }

    /// Returns the list of GDTF file paths referenced by this showfile.
    ///
    /// The returned slice contains the absolute or relative file paths that were
    /// discovered when the showfile was loaded (or that were set prior to
    /// saving). These point to the original GDTF source files.
    pub fn gdtf_file_paths(&self) -> &[PathBuf] {
        &self.gdtf_file_paths
    }

    /// Returns a reference to the [`Config`] section of the showfile.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Returns a reference to the [`Patch`] section of the showfile.
    pub fn patch(&self) -> &Patch {
        &self.patch
    }

    /// Returns a reference to the [`Protocols`] section of the showfile.
    pub fn protocols(&self) -> &Protocols {
        &self.protocols
    }
}

/// Used for giving elements a visual label in the showfile.
#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct Label(String);

impl Label {
    /// Creates a new [`Label`].
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A unique identifier, consisting of a namespace and a value.
/// The namespace represents a component. For Zeevonk, this will be 'zeevonk',
/// but for an external program this could be different.
/// The value is a unique thing within that namespace.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize)]
pub struct Identifier {
    /// The namespace represents a component. For Zeevonk, this will be 'zeevonk',
    /// but for an external program this could be different.
    namespace: String,
    /// The value is a unique thing within a namespace.
    value: String,
}

impl Identifier {
    /// Create a new Identifier, validating namespace and value.
    pub fn new(namespace: impl AsRef<str>, value: impl AsRef<str>) -> anyhow::Result<Self> {
        let mut id = Identifier::default();
        id.set_namespace(namespace.as_ref())?;
        id.set_value(value.as_ref())?;
        Ok(id)
    }

    /// Returns the namespace.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Returns the value.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Sets the namespace if valid. Returns Ok(()) if successful, Err otherwise.
    pub fn set_namespace(&mut self, namespace: &str) -> anyhow::Result<()> {
        if Self::is_valid(namespace) {
            self.namespace = namespace.to_string();
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "invalid namespace: must be non-empty, lowercase, alphanumeric or '_'"
            ))
        }
    }

    /// Sets the value if valid. Returns Ok(()) if successful, Err otherwise.
    pub fn set_value(&mut self, value: &str) -> anyhow::Result<()> {
        if Self::is_valid(value) {
            self.value = value.to_string();
            Ok(())
        } else {
            Err(anyhow::anyhow!("invalid value: must be non-empty, lowercase, alphanumeric or '_'"))
        }
    }

    /// Validate that a string is lowercase, alphanumeric or '_'
    fn is_valid(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.value)
    }
}
