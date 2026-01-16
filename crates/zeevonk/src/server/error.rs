//! Error types and result aliases for this crate.

use crate::server::output;

/// A specialized `Result` type for operations that can return an [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error occurred while handling output.
    #[error("output error: {0}")]
    OutputError(#[from] output::Error),
}
