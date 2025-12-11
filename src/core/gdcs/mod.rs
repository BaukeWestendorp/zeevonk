use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use gdtf::dmx_mode::DmxMode;
use gdtf::fixture_type::FixtureType;
use uuid::Uuid;

use crate::core::dmx::{Address, Channel, Multiverse, UniverseId};
use crate::core::gdcs;
use crate::core::gdcs::fixture::builder::FixtureBuilder;
use crate::core::gdcs::resolver::Resolver;
use crate::core::showfile::Showfile;

pub use attr::*;
pub use error::*;
pub use fixture::*;
pub use util::*;

mod attr;
mod fixture;

mod error;
mod resolver;
mod util;

/// The Generalized DMX Control System (GDCS) manages GDTF fixture types
/// and user-defined fixtures, to make it easy to work with any kind of fixture,
/// while still providing a way to use all attributes a fixture contains.
#[derive(Debug)]
pub struct GeneralizedDmxControlSystem {
    fixture_types: HashMap<Uuid, FixtureType>,
    dmx_mode_channel_counts: HashMap<(Uuid, String), u32>,

    fixtures: HashMap<FixturePath, Fixture>,
    fixture_roots: HashSet<FixtureId>,

    resolved_multiverse: Multiverse,
    channel_function_values: HashMap<(FixturePath, Attribute), ClampedValue>,
}

impl GeneralizedDmxControlSystem {
    /// Create a new, empty GDCS.
    pub fn new() -> Self {
        Self {
            fixture_types: HashMap::new(),
            dmx_mode_channel_counts: HashMap::new(),

            fixtures: HashMap::new(),
            fixture_roots: HashSet::new(),

            channel_function_values: HashMap::new(),
            resolved_multiverse: Multiverse::new(),
        }
    }

    /// Registers GDTF files and fixtures from the given [`Showfile`].
    pub fn insert_showfile_data(&mut self, showfile: &Showfile) -> Result<(), gdcs::Error> {
        for gdtf_file_path in showfile.gdtf_file_paths() {
            self.register_gdtf_file(gdtf_file_path)?;
        }

        for fixture in showfile.patch().fixtures() {
            self.register_fixture(
                fixture.id(),
                fixture.label().to_string(),
                fixture.address(),
                fixture.kind().gdtf_fixture_type_id(),
                fixture.kind().gdtf_dmx_mode().to_string(),
            )?;
        }

        Ok(())
    }

    /// Returns all registered GDTF fixture types.
    pub fn gdtf_fixture_types(&self) -> impl Iterator<Item = (&Uuid, &FixtureType)> {
        self.fixture_types.iter()
    }

    /// Returns the channel count for a given fixture type and DMX mode name.
    ///
    /// The DMX mode name must match one that was present in the GDTF fixture
    /// type that was previously registered. If no entry exists, `None` is
    /// returned.
    pub fn dmx_mode_channel_count(&self, fixture_type_id: Uuid, dmx_mode: String) -> Option<u32> {
        self.dmx_mode_channel_counts.get(&(fixture_type_id, dmx_mode)).copied()
    }

    /// Returns all instantiated fixtures.
    pub fn fixtures(&self) -> impl Iterator<Item = &Fixture> {
        self.fixtures.values()
    }

    /// Returns all root fixtures.
    ///
    /// Root fixtures are those whose `FixturePath` has only one part. Top-level fixtures.
    pub fn root_fixtures(&self) -> impl Iterator<Item = &Fixture> {
        self.fixtures.values().filter(|fixture| fixture.path().is_root_fixture())
    }

    /// Register a GDTF (.gdtf) file into the system.
    ///
    /// Returns an error if the file cannot be opened or parsed.
    pub fn register_gdtf_file(&mut self, path: &Path) -> Result<(), gdcs::Error> {
        let file = fs::File::open(path)?;
        let gdtf_file = gdtf::GdtfFile::new(file)?;

        for fixture_type in gdtf_file.description.fixture_types {
            let fixture_type_id = fixture_type.fixture_type_id;

            for dmx_mode in &fixture_type.dmx_modes {
                let Some(dmx_mode_name) = dmx_mode.name.clone() else {
                    log::error!("no name found for dmx mode");
                    continue;
                };

                let channel_count = calculate_channel_count_for_dmx_mode(&fixture_type, dmx_mode);

                self.dmx_mode_channel_counts
                    .insert((fixture_type_id, dmx_mode_name.to_string()), channel_count);
            }

            self.fixture_types.insert(fixture_type_id, fixture_type);
        }

        Ok(())
    }

    /// Register a fixture (root) and its child fixtures into the system.
    pub fn register_fixture(
        &mut self,
        root_id: FixtureId,
        name: String,
        address: Address,
        gdtf_fixture_type_id: Uuid,
        gdtf_dmx_mode: String,
    ) -> Result<(), gdcs::Error> {
        if self.fixture_roots.contains(&root_id) {
            return Err(gdcs::Error::FixtureAlreadyExists(root_id.as_u32()));
        }

        if !self.address_available(&address) {
            return Err(gdcs::Error::AddressAlreadyMapped(address));
        }

        let Some(gdtf_fixture_type) = self.gdtf_fixture_type(gdtf_fixture_type_id) else {
            return Err(gdcs::Error::FixtureTypeNotFound(gdtf_fixture_type_id));
        };

        let Some(gdtf_dmx_mode) = gdtf_fixture_type.dmx_mode(&gdtf_dmx_mode) else {
            return Err(gdcs::Error::InvalidDmxMode(gdtf_dmx_mode));
        };

        let fixtures =
            FixtureBuilder::new(root_id, name, address, gdtf_fixture_type, gdtf_dmx_mode)
                .build_fixture_tree()?;

        self.fixture_roots.insert(root_id);
        for fixture in fixtures {
            self.fixtures.insert(fixture.path, fixture);
        }

        Ok(())
    }

    /// Get a registered GDTF fixture type by its UUID.
    pub fn gdtf_fixture_type(&self, fixture_type_id: Uuid) -> Option<&FixtureType> {
        self.fixture_types.get(&fixture_type_id)
    }

    /// Get a fixture by its path.
    pub fn fixture(&self, path: &FixturePath) -> Option<&Fixture> {
        self.fixtures.get(path)
    }

    /// Get the root fixture for a given root id.
    pub fn root_fixture(&self, root_id: FixtureId) -> Option<&Fixture> {
        self.fixtures.get(&FixturePath::new(root_id))
    }

    /// Return all registered fixture paths.
    pub fn fixture_paths(&self) -> impl Iterator<Item = FixturePath> {
        self.fixtures.keys().copied()
    }

    /// Get the resolved multiverse.
    ///
    /// The multiverse represents the final DMX channel values after the
    /// resolver has been run with the current fixtures and unresolved values.
    pub fn resolved_multiverse(&self) -> &Multiverse {
        &self.resolved_multiverse
    }

    /// Run the resolver to produce an updated resolved multiverse
    /// that can be accessed with [Self::resolved_multiverse].
    pub fn resolve(&mut self) {
        self.resolved_multiverse = Resolver::new(self).resolve();
    }

    /// Get all unresolved channel function values.
    pub fn unresolved_values(&self) -> &HashMap<(FixturePath, Attribute), ClampedValue> {
        &self.channel_function_values
    }

    /// Check whether a DMX address is available (not occupied by another fixture).
    pub fn address_available(&self, address: &Address) -> bool {
        !self.fixtures().into_iter().any(|f| {
            let channel_count = self
                .dmx_mode_channel_count(f.gdtf_fixture_type_id, f.gdtf_dmx_mode.clone())
                .expect("channel count should be known");

            let low = f.base_address().to_absolute();
            let high = low + channel_count as u32;
            let channel_range = low..high;
            channel_range.contains(&address.to_absolute())
        })
    }

    /// Check whether a fixture path is available (not already used).
    pub fn path_available(&self, path: &FixturePath) -> bool {
        !self.fixtures().into_iter().any(|f| f.path() == *path)
    }

    /// Check whether a fixture would collide by path or address.
    pub fn fixture_collides(&self, path: &FixturePath, address: &Address) -> bool {
        !self.path_available(path) || !self.address_available(address)
    }

    /// Set (or update) an unresolved channel function value for a fixture attribute.
    ///
    /// It will be used by the resolver the next time [Self::resolve()] is called.
    pub fn set_channel_function_value(
        &mut self,
        fixture_path: FixturePath,
        attribute: Attribute,
        value: impl Into<ClampedValue>,
    ) {
        self.channel_function_values.insert((fixture_path, attribute), value.into());
    }
}

/// Compute how many DMX channels a given DMX mode uses for a fixture type.
fn calculate_channel_count_for_dmx_mode(fixture_type: &FixtureType, dmx_mode: &DmxMode) -> u32 {
    // To calculate the channel count for a certain dmx mode, we just create a temporary fixture, and find
    // the highest used address for that fixture.

    let start_address = Address::new(UniverseId::new(1).unwrap(), Channel::new(1).unwrap());

    let fixtures = FixtureBuilder::new(
        FixtureId::new(1).unwrap(),
        "F".to_string(),
        start_address,
        fixture_type,
        dmx_mode,
    )
    .build_fixture_tree()
    .expect("should build temporary fixture");

    let mut highest_address = start_address;

    for fixture in fixtures {
        for (_, cf) in fixture.channel_functions() {
            match cf.kind() {
                FixtureChannelFunctionKind::Physical { addresses } => {
                    if let Some(&highest_cf_address) = addresses.iter().max() {
                        if highest_cf_address > highest_address {
                            highest_address = highest_cf_address;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    highest_address.to_absolute() - start_address.to_absolute() + 1
}
