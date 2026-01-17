#![warn(missing_docs)]

//! Zeevonk is a modular lighting control system, consisting of a server and two kinds of client.
//!
//! <div class="warning">
//!
//! **Warning**
//!
//! Zeevonk is currently in early development. APIs, features, and behavior may change frequently and without notice.
//! It is not yet recommended for production use.
//!
//! </div>
//!
//! # Components
//!
//! - [Server](crate::server): Manages clients, resolves attributes, and sends DMX.
//! - [Processor Client](crate::client::processor): Generates and sends attribute values to the server.
//! - [Controller Client](crate::client::controller): Originates triggers sent to the server.
//!
//! See the respective module documentation for details.

pub mod attr;
pub mod error;
pub mod ident;
pub mod packet;
pub mod project;
pub mod trigger;
pub mod value;

#[cfg(any(feature = "client-processor", feature = "client-controller"))]
pub mod client;
#[cfg(feature = "server")]
pub mod server;

pub use error::{Error, Result};
