//! # Zeevonk
//!
//! Zeevonk is a modular lighting control system, consisting of a server and multiple kinds of client.

#![warn(missing_docs)]

pub mod core;
pub use core::*;

#[cfg(any(feature = "client-processor"))]
pub mod client;
#[cfg(feature = "server")]
pub mod server;
