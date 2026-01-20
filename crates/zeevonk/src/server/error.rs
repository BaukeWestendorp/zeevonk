//! Error types and result aliases for Zeevonk.

use std::io;

use crate::ident::Identifier;
use crate::server::output;

/// A [`Result`] wrapper for Zeevonk errors.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in this crate.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Wraps [`io::Error`].
    #[error("i/o error: {0}")]
    Io(#[from] io::Error),

    /// An error occurred while handling output.
    #[error("output error: {0}")]
    OutputError(#[from] output::Error),

    /// A client was not found with the provided identifier.
    #[error("client not found with identifier '{0}'")]
    ClientNotFound(Identifier),

    /// Failed to decode packet.
    #[error("packet decoding failed")]
    PacketDecodingFailed,
}
