//! Contains the different kind of client implementations.

#[cfg(feature = "client-processor")]
pub mod processor;

pub mod error;

pub use error::{Error, Result};
