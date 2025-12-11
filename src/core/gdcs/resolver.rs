//! See [Resolver] for more information.

use crate::core::dmx::Multiverse;
use crate::core::gdcs::attr::Attribute;
use crate::core::gdcs::fixture::{
    Fixture, FixtureChannelFunction, FixtureChannelFunctionKind, FixturePath, Relation,
    RelationKind,
};
use crate::core::gdcs::{
    ClampedValue, GeneralizedDmxControlSystem, clamped_value_to_address_values,
};

/// Resolver for translating GDCS state into a physical DMX multiverse.
///
/// The resolver walks the fixtures, computes the effective value for
/// each fixture channel function, and writes the corresponding bytes into a
/// [dmx::Multiverse]. Virtual channel functions (those driven by relations)
/// are resolved by deferring relation writes until all fixtures have been
/// examined. This allows follower relations (multiply or override) to be
/// resolved against the master's computed values.
pub struct Resolver<'gdcs> {
    /// Reference to the GDCS to resolve.
    gdcs: &'gdcs GeneralizedDmxControlSystem,
    /// The multiverse that will be populated with DMX values.
    multiverse: Multiverse,
    /// Relations whose writes are deferred until after the initial fixture
    /// pass. Each entry contains the relation and the resolved value to apply.
    /// This is needed for resolving virtual channels.
    deferred_relations: Vec<(&'gdcs Relation, ClampedValue)>,
}

impl<'gdcs> Resolver<'gdcs> {
    /// Create a new resolver.
    pub fn new(gdcs: &'gdcs GeneralizedDmxControlSystem) -> Self {
        Self { gdcs, multiverse: gdcs.defaulted_multiverse.clone(), deferred_relations: Vec::new() }
    }

    /// Perform resolution and return the populated multiverse.
    pub fn resolve(mut self) -> Multiverse {
        for fixture in self.gdcs.fixtures.values() {
            self.resolve_fixture(fixture);
        }

        let deferred_writes = std::mem::take(&mut self.deferred_relations);
        for (relation, value) in deferred_writes {
            let Some(channel_function) = self
                .gdcs
                .fixture(&relation.fixture_path())
                .and_then(|f| f.channel_function(relation.attribute()))
            else {
                continue;
            };

            self.set_channel_function_value(channel_function, value);
        }

        self.multiverse
    }

    /// Resolve all channel functions of a single fixture.
    fn resolve_fixture(&mut self, fixture: &'gdcs Fixture) {
        for (attribute, channel_function) in fixture.channel_functions() {
            if let Some(value) = self.get_channel_function_value(fixture.path(), attribute) {
                self.set_channel_function_value(channel_function, value);
            }
        }
    }

    /// Determines the value for a specific channel function explicitly present in the GDCS's unresolved values map.
    fn get_channel_function_value(
        &self,
        fixture_path: FixturePath,
        attribute: &Attribute,
    ) -> Option<ClampedValue> {
        self.gdcs.unresolved_values().get(&(fixture_path, attribute.clone())).copied()
    }

    /// Apply a computed value to a channel function.
    ///
    /// For physical channel functions, converts the `ClampedValue` to the
    /// appropriate byte sequence and writes it into the multiverse at the
    /// configured addresses.
    ///
    /// For virtual channel functions, evaluates relations and defers the
    /// actual writes so that they can be applied after the initial pass.
    fn set_channel_function_value(
        &mut self,
        channel_function: &'gdcs FixtureChannelFunction,
        value: ClampedValue,
    ) {
        match channel_function.kind() {
            FixtureChannelFunctionKind::Physical { addresses } => {
                let values = clamped_value_to_address_values(addresses, value);
                for (address, value) in values {
                    self.multiverse.set_value(&address, value);
                }
            }
            FixtureChannelFunctionKind::Virtual { relations } => {
                for relation in relations {
                    match *relation.kind() {
                        RelationKind::Multiply => {
                            if let Some(follower_value) = self.get_channel_function_value(
                                relation.fixture_path(),
                                relation.attribute(),
                            ) {
                                let new_value =
                                    ClampedValue::new(follower_value.as_f32() * value.as_f32());
                                self.defer_relation_resolution(relation, new_value);
                            };
                        }
                        RelationKind::Override => {
                            self.defer_relation_resolution(relation, value);
                        }
                    }
                }
            }
        }
    }

    /// Queue a relation write to be applied after the initial resolution pass.
    ///
    /// Deferring relation resolutions ensures that master values are computed
    /// before followers are written.
    fn defer_relation_resolution(&mut self, relation: &'gdcs Relation, value: ClampedValue) {
        self.deferred_relations.push((relation, value));
    }
}
