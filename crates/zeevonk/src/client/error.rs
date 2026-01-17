//! Error types and result aliases for Zeevonk clients.

/// A [`Result`] wrapper for Zeevonk client errors.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur in a Zeevonk client.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The connection with the server has been closed.
    #[error("the connection with the server has been closed")]
    ConnectionClosed,
}
