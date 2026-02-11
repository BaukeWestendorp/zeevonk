//! Error types.

use std::io;

use uuid::Uuid;

/// Convenient alias for results returned by this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors produced by this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Wraps [`io::Error`].
    #[error("i/o error: {0}")]
    Io(#[from] io::Error),

    /// An invalid fixture id was encountered (e.g. zero).
    #[error("invalid fixture id")]
    InvalidFixtureId,
    /// Fixture id string was empty.
    #[error("empty fixture id")]
    EmptyFixtureId,
    /// Fixture id had too many parts.
    #[error("fixture id has too many parts (max {0})")]
    FixtureIdTooLong(usize),

    /// The requested fixture type was not found.
    #[error("fixture type not found: {id}")]
    FixtureTypeNotFound {
        /// The UUID of the fixture type that was not found.
        id: Uuid,
    },
    /// The requested DMX mode was not found for the given fixture type.
    #[error("dmx mode not found")]
    DmxModeNotFound,
    /// The root geometry was not found for the given fixture type and DMX mode.
    #[error("root geometry not found")]
    RootGeometryNotFound,

    /// An error occurred while handling output.
    #[error("output error: {0}")]
    OutputError(#[from] crate::output::Error),
}
