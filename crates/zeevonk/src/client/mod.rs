//! Contains the different kind of client implementations.
//!
//! - For processor client details, see [`processor`].
//! - For controller client details, see [`controller`].

#[cfg(feature = "client-controller")]
pub mod controller;
#[cfg(feature = "client-processor")]
pub mod processor;

pub mod error;

pub use error::{Error, Result};
