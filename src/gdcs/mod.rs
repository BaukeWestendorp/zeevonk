use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use gdtf::dmx_mode::DmxMode;
use gdtf::fixture_type::FixtureType;
use uuid::Uuid;

use crate::dmx::{Address, Channel, Multiverse, UniverseId};
use crate::gdcs::attr::Attribute;
use crate::gdcs::fixture::builder::FixtureBuilder;
use crate::gdcs::fixture::{Fixture, FixtureChannelFunctionKind, FixtureId, FixturePath};
use crate::gdcs::resolver::Resolver;

pub use error::GdcsError;
pub use util::ClampedValue;

pub mod attr;
pub mod fixture;

mod error;
mod resolver;
mod util;

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

    pub fn gdtf_fixture_types(&self) -> impl Iterator<Item = (&Uuid, &FixtureType)> {
        self.fixture_types.iter()
    }

    pub fn dmx_mode_channel_count(&self, fixture_type_id: Uuid, dmx_mode: String) -> Option<u32> {
        self.dmx_mode_channel_counts.get(&(fixture_type_id, dmx_mode)).copied()
    }

    pub fn fixtures(&self) -> impl IntoIterator<Item = &Fixture> {
        self.fixtures.values()
    }

    pub fn root_fixtures(&self) -> impl IntoIterator<Item = &Fixture> {
        self.fixtures.values().filter(|fixture| fixture.path().is_root_fixture())
    }

    pub fn register_gdtf_file(&mut self, path: &Path) -> Result<(), GdcsError> {
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

    pub fn register_fixture(
        &mut self,
        root_id: FixtureId,
        name: String,
        address: Address,
        gdtf_fixture_type_id: Uuid,
        gdtf_dmx_mode: String,
    ) -> Result<(), GdcsError> {
        if self.fixture_roots.contains(&root_id) {
            return Err(GdcsError::FixtureAlreadyExists(root_id.as_u32()));
        }

        if !self.address_available(&address) {
            return Err(GdcsError::AddressAlreadyMapped(address));
        }

        let Some(gdtf_fixture_type) = self.gdtf_fixture_type(gdtf_fixture_type_id) else {
            return Err(GdcsError::FixtureTypeNotFound(gdtf_fixture_type_id));
        };

        let Some(gdtf_dmx_mode) = gdtf_fixture_type.dmx_mode(&gdtf_dmx_mode) else {
            return Err(GdcsError::InvalidDmxMode(gdtf_dmx_mode));
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

    pub fn gdtf_fixture_type(&self, fixture_type_id: Uuid) -> Option<&FixtureType> {
        self.fixture_types.get(&fixture_type_id)
    }

    pub fn fixture(&self, path: &FixturePath) -> Option<&Fixture> {
        self.fixtures.get(path)
    }

    pub fn root_fixture(&self, root_id: FixtureId) -> Option<&Fixture> {
        self.fixtures.get(&FixturePath::new(root_id))
    }

    pub fn fixture_paths(&self) -> impl IntoIterator<Item = FixturePath> {
        self.fixtures.keys().copied()
    }

    pub fn resolved_multiverse(&self) -> &Multiverse {
        &self.resolved_multiverse
    }

    pub fn resolve(&mut self) {
        self.resolved_multiverse = Resolver::new(self).resolve();
    }

    pub fn unresolved_values(&self) -> &HashMap<(FixturePath, Attribute), ClampedValue> {
        &self.channel_function_values
    }

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

    pub fn path_available(&self, path: &FixturePath) -> bool {
        !self.fixtures().into_iter().any(|f| f.path() == *path)
    }

    pub fn fixture_collides(&self, path: &FixturePath, address: &Address) -> bool {
        !self.path_available(path) || !self.address_available(address)
    }

    pub fn set_channel_function_value(
        &mut self,
        fixture_path: FixturePath,
        attribute: Attribute,
        value: impl Into<ClampedValue>,
    ) {
        self.channel_function_values.insert((fixture_path, attribute), value.into());
    }
}

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
