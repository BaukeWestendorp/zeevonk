//! # Zeevonk Core
//!
//! Core types and logic for the Zeevonk lighting control system.
//!
//! This crate provides the foundational data structures and utilities for working with fixtures,
//! attributes, DMX patching, and value management. It is intended to be used by higher-level
//! components of the Zeevonk like the server and the different clients.

pub mod attr;
pub mod error;
pub mod ident;
pub mod packet;
pub mod project;
pub mod trigger;
pub mod value;

pub use error::{Error, Result};
