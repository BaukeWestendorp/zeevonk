/// DMX utilities.
pub mod dmx;

/// GDCS.
pub mod gdcs;
/// Showfile management.
pub mod showfile;

pub(crate) mod packet;
pub(crate) mod util;

/// The default port used for network communication.
pub const DEFAULT_PORT: u16 = 7334;
