#![warn(missing_docs)]

//! Zeevonk is a modular lighting control system.
//! The Zeevonk server is a hub for multiple clients that can update attribute values,
//! which the server then will output to the configured DMX protocols.
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
//! - [Client](crate::client): Can generate and send attribute values to the server, or talk to other clients using [`Trigger`][trigger::Trigger]s.
//!
//! See the respective module documentation for details.

pub mod attr;
pub mod error;
pub mod ident;
pub mod packet;
pub mod project;
pub mod trigger;
pub mod value;

#[cfg(any(feature = "client"))]
pub mod client;
#[cfg(feature = "server")]
pub mod server;

pub use error::{Error, Result};
