#![warn(missing_docs)]

//! # Zeevonk.

/// DMX utilities.
pub mod dmx;
/// The `Engine` is responsible for managing the main runtime state of the application.
pub mod engine;
/// Packet handling.
pub mod packet;
/// Showfile management.
pub mod showfile;

/// The default port used for network communication.
pub const DEFAULT_PORT: u16 = 7334;
