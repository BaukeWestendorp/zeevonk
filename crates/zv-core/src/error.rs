//! Error types.

use std::io;

/// Convenient alias for results returned by this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors produced by this crate.
///
/// Each variant represents a different class of failure that can occur while
/// performing crate operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Wraps [`io::Error`].
    #[error("i/o error: {0}")]
    Io(#[from] io::Error),

    /// An invalid identifier was encountered.
    #[error("invalid identifier")]
    InvalidIdentifier,

    /// An invalid fixture id was encountered (e.g. zero).
    #[error("invalid fixture id")]
    InvalidFixtureId,

    /// Fixture id string was empty.
    #[error("empty fixture id")]
    EmptyFixtureId,

    /// Fixture id had too many parts.
    #[error("fixture id has too many parts (max {0})")]
    FixtureIdTooLong(usize),
}
