//! Contains the different kind of client implementations.
//!
//! # The Processor Client
//!
//! **Note:** The `client-processor` feature must be enabled to use a processor client in your code.
//!
//! A processor client is responsible for generating high-level [GDTF](https://gdtf.eu) attribute values for
//! specific fixtures and sending them to the server.
//!
//! Typical responsibilities of a processor client include:
//! - Subscribing to [triggers](crate::trigger).
//! - Mapping [triggers](crate::trigger) to fixture/attribute targets and resolving which attributes should change.
//! - Calculating or interpolating attribute values (effects, fades, curves, color mixing, etc.).
//! - Sending attribute updates to the server for DMX output.
//! - Maintaining local state and managing transitions (so updates are smooth and deterministic).
//!
//! # The Controller Client
//!
//! **Note:** The `client-controller` feature must be enabled to use a controller client in your code.
//!
//! A controller client is the origin of [triggers](crate::trigger).
//!
//! Typical responsibilities of a controller client include:
//! - Sending [triggers](crate::trigger) (MIDI, OSC, button presses, fader changes, cue selections, etc.) to the server.
//!
//! # Examples
//!
//! FIXME: Add examples.

#[cfg(feature = "client-controller")]
pub mod controller;
#[cfg(feature = "client-processor")]
pub mod processor;

pub mod error;

pub use error::{Error, Result};
