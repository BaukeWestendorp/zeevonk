#![warn(missing_docs)]

//! # Zeevonk.

/// Commonly used types for Zeevonk.
pub mod prelude;

/// Zeevonk client.
pub mod client;
/// Zeevonk server.
pub mod server;

mod packet;

/// DMX utilities.
pub mod dmx;

/// GDCS.
pub mod gdcs;
/// Showfile management.
pub mod showfile;

mod util;

/// The default port used for network communication.
pub const DEFAULT_PORT: u16 = 7334;
