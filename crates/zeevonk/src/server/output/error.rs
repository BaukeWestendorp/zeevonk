#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("ftdi error: {0:?}")]
    FtdiError(#[from] libftd2xx::FtStatus),

    #[error("timed out")]
    Timeout,
}
