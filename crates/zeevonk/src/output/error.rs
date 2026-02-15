use crate::output::protocols::sacn;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("ftdi error: {0}")]
    FtdiError(#[from] libftd2xx::FtStatus),

    #[error("sacn error: {0}")]
    SacnError(#[from] sacn::source::SourceError),

    #[error("timed out")]
    Timeout,

    #[error("{message}")]
    Other { message: String },
}

impl Error {
    pub fn other(message: impl Into<String>) -> Self {
        Self::Other { message: message.into() }
    }
}
