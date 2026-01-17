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
//!
//! # The Server
//!
//! **Note:** The `server` feature must be enabled to start and manage a server from your
//! own code. If you prefer a ready-made program instead of embedding a server, use the standalone
//! zeevonk command-line tool. See the [`zeevonk` CLI](FIXME) for installation and usage details.
//!
//! The Zeevonk server is a hub for managing clients. It has a few essential responsibilities:
//! - Receiving [triggers](crate::trigger) from [controller clients](crate::client::controller) and routing them
//!   to the correct [processor clients](crate::client::processor).
//! - Receiving attribute updates from [processor clients](crate::client::processor)
//!   and converting them to DMX output.
//! - Sending DMX output over [various protocols](crate::project::dmx_output)
//!   like [sACN](crate::project::definition::dmx_output::DmxOutputInstanceDefinition)
//!   or [Entecc Open DMX](crate::project::definition::dmx_output::DmxOutputInstanceDefinition).
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
