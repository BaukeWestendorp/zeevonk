#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

/// Commonly used types for Zeevonk.
///
/// The prelude is intended to be used with `use zeevonk::prelude::*`.
pub mod prelude {
    pub use crate::client::{Client, ProcessorContext};
    pub use crate::core::gdcs::{Attribute, ClampedValue, Fixture};
}

/// Modules that are both used in the server and the client.
pub mod core;

/// Zeevonk client.
pub mod client;
/// Zeevonk server.
pub mod server;
