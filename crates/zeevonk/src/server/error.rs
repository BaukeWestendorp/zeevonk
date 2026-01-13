use std::io;
use std::path::PathBuf;

use uuid::Uuid;

use crate::server::output::sacn::SourceError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to open GDTF file at {path:?}: {source}")]
    GdtfFileOpenError { path: PathBuf, source: io::Error },

    #[error("Failed to parse GDTF file at {path:?}: {source}")]
    GdtfFileParseError { path: PathBuf, source: gdtf::GdtfError },

    #[error("Fixture type not found: {id}")]
    FixtureTypeNotFound { id: Uuid },

    #[error("DMX mode '{mode}' not found for fixture type {fixture_type_id}")]
    DmxModeNotFound { mode: String, fixture_type_id: Uuid },

    #[error(
        "Root geometry not found for fixture type {fixture_type_id}, DMX mode: {dmx_mode_name:?}"
    )]
    RootGeometryNotFound { fixture_type_id: Uuid, dmx_mode_name: String },

    #[error("Failed to build fixture tree: {source}")]
    FixtureTreeBuildError { source: Box<Error> },

    #[error("sACN source erorr: {0}")]
    SourceError(#[from] SourceError),
}
