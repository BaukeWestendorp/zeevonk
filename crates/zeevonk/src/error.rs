use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("i/o error: {0}")]
    Io(#[from] io::Error),

    #[cfg(feature = "server")]
    #[error("server error: {message}")]
    Server { message: String },

    #[error("{message}")]
    Other { message: String },
}

impl Error {
    #[cfg(feature = "server")]
    pub(crate) fn server(message: impl Into<String>) -> Self {
        Self::Server { message: message.into() }
    }

    pub(crate) fn other(message: impl Into<String>) -> Self {
        Self::Other { message: message.into() }
    }
}
