pub use error::*;

pub mod attr;
pub mod dmx;
pub mod packet;
pub mod showfile;
pub mod state;
pub mod value;

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "server")]
pub mod server;

mod error;

pub const DEFAULT_PORT: u16 = 7334;
