#![warn(missing_docs)]

//! # Zeevonk.

/// The [Client][client::Client] is the main API for interacting with the zeevonk server.
pub mod client;

/// DMX utilities.
pub mod dmx;
/// The [Engine][engine::Engine] is responsible for managing the main runtime state of the application.
mod engine;
/// Generalized DMX Control System.
pub mod gdcs;
/// Packet handling.
mod packet;
/// Showfile management.
mod showfile;

/// The default port used for network communication.
pub const DEFAULT_PORT: u16 = 7334;
