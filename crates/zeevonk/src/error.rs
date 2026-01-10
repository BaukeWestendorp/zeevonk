use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("i/o error: {0}")]
    Io(#[from] io::Error),

    #[error("{message}")]
    Other { message: String },
}

impl Error {
    pub(crate) fn other(message: impl Into<String>) -> Self {
        Self::Other { message: message.into() }
    }
}
