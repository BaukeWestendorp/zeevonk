use std::io;

use uuid::Uuid;

use crate::dmx::Address;
use crate::gdcs::attr::Attribute;
use crate::gdcs::fixture::FixturePath;

#[derive(Debug, thiserror::Error)]
pub enum GdcsError {
    /// An I/O error.
    #[error("i/o error")]
    Io(#[from] io::Error),
    /// An error from the GDTF crate.
    #[error("gdtf error")]
    Gdtf(#[from] gdtf::GdtfError),
    /// Error while parsing an attribute.
    #[error("failed to parse attribute")]
    AttributeParseError,
    /// A fixture with the given ID already exists.
    #[error("fixture with id {0} already exists")]
    FixtureAlreadyExists(u32),
    /// The address is already mapped.
    #[error("address {0} is already mapped")]
    AddressAlreadyMapped(Address),
    /// The requested attribute was not found for the fixture.
    #[error("attribute {0} not found for fixture")]
    AttributeNotFoundForFixture(Attribute),
    /// The requested fixture type was not found.
    #[error("fixture type with id {0} not found")]
    FixtureTypeNotFound(Uuid),
    /// The provided fixture id is invalid.
    #[error("invalid fixture id: {0}")]
    InvalidFixtureId(u32),
    /// The provided DMX mode is invalid.
    #[error("invalid dmx mode: {0}")]
    InvalidDmxMode(String),
    /// The requested (sub)fixture was not found.
    #[error("fixture not found: {0}")]
    FixtureNotFound(FixturePath),
    /// The fixture address is already taken.
    #[error("fixture address already taken: {0}")]
    FixtureAddressAlreadyTaken(Address),
    /// The fixture does not have the requested attribute
    #[error("attribute {0} not found for fixture")]
    InvalidAttributeForFixture(Attribute),
    /// Could not parse fixture path
    #[error("failed to parse fixture path: {message}")]
    FailedToParseFixturePath { message: String },
}
