#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

/// Commonly used types for Zeevonk.
///
/// The prelude is intended to be used with `use zeevonk::prelude::*`.
pub mod prelude {
    pub use crate::client::{Client, ProcessorContext};
    pub use crate::core::gdcs::{Attribute, ClampedValue, Fixture};
}

/// Modules that are both used in the [server] and the [client].
pub mod core;

/// A client that can communicate with a Zeevonk [server] (e.g. sending and receiving triggers or setting attribute values).
pub mod client;
/// The Zeevonk server serves as a hub to connect multiple clients together and generating DMX output over various protocols.
pub mod server;
