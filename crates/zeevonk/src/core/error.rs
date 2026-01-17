//! Error types.

use std::io;

use uuid::Uuid;

/// Convenient alias for results returned by this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors produced by this crate.
///
/// Each variant represents a different class of failure that can occur while
/// performing crate operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Wraps [`io::Error`].
    #[error("i/o error: {0}")]
    Io(#[from] io::Error),

    /// An invalid identifier was encountered.
    #[error("invalid identifier")]
    InvalidIdentifier,
    /// An invalid fixture id was encountered (e.g. zero).
    #[error("invalid fixture id")]
    InvalidFixtureId,
    /// Fixture id string was empty.
    #[error("empty fixture id")]
    EmptyFixtureId,
    /// Fixture id had too many parts.
    #[error("fixture id has too many parts (max {0})")]
    FixtureIdTooLong(usize),

    /// The requested fixture type was not found.
    #[error("fixture type not found: {id}")]
    FixtureTypeNotFound {
        /// The UUID of the fixture type that was not found.
        id: Uuid,
    },
    /// The requested DMX mode was not found for the given fixture type.
    #[error("dmx mode not found: {mode} (fixture type id: {fixture_type_id})")]
    DmxModeNotFound {
        /// The name of the DMX mode that was not found.
        mode: String,
        /// The UUID of the fixture type for which the DMX mode was not found.
        fixture_type_id: Uuid,
    },
    /// The root geometry was not found for the given fixture type and DMX mode.
    #[error(
        "root geometry not found (fixture type id: {fixture_type_id}, dmx mode: {dmx_mode_name})"
    )]
    RootGeometryNotFound {
        /// The UUID of the fixture type for which the root geometry was not found.
        fixture_type_id: Uuid,
        /// The name of the DMX mode for which the root geometry was not found.
        dmx_mode_name: String,
    },

    /// Received an invalid packet.
    #[error("received an invalid packet: {message}")]
    InvalidPacket {
        /// Information about the invalid packet.
        message: String,
    },
}
