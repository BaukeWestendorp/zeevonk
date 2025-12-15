use tokio::sync::RwLock;

use crate::attr::Attribute;
use crate::dmx::Multiverse;
use crate::packet::AttributeValues;
use crate::server::Inner;
use crate::state::State;
use crate::state::fixture::{
    FixtureChannelFunction, FixtureChannelFunctionKind, FixturePath, Relation, RelationKind,
};
use crate::value::ClampedValue;

impl Inner {
    pub async fn resolve_values(&self) {
        Resolver::new(&self.pending_attribute_values, &self.state, &self.output_multiverse)
            .resolve()
            .await;
    }
}

/// Resolver for translating GDCS state into a physical DMX multiverse.
///
/// The resolver walks the fixtures, computes the effective value for
/// each fixture channel function, and writes the corresponding bytes into a
/// [dmx::Multiverse]. Virtual channel functions (those driven by relations)
/// are resolved by deferring relation writes until all fixtures have been
/// examined. This allows follower relations (multiply or override) to be
/// resolved against the master's computed values.
struct Resolver<'a> {
    attribute_values: &'a RwLock<AttributeValues>,
    state: &'a RwLock<State>,
    multiverse: &'a RwLock<Multiverse>,

    /// Relations whose writes are deferred until after the initial fixture
    /// pass. Each entry contains the relation and the resolved value to apply.
    /// This is needed for resolving virtual channels.
    deferred_relations: Vec<(Relation, ClampedValue)>,
}

impl<'a> Resolver<'a> {
    /// Create a new resolver.
    pub fn new(
        attribute_values: &'a RwLock<AttributeValues>,
        state: &'a RwLock<State>,
        multiverse: &'a RwLock<Multiverse>,
    ) -> Self {
        Self { attribute_values, state, multiverse, deferred_relations: Vec::new() }
    }

    /// Perform resolution and return the populated multiverse.
    pub async fn resolve(mut self) {
        // Collect fixture paths while holding the state read lock briefly.
        let fixture_paths: Vec<FixturePath> = {
            let state = self.state.read().await;
            state.patch.fixtures.keys().cloned().collect()
        };

        // Resolve each fixture independently. We don't hold any long-lived
        // state guards across await points.
        for fixture_path in fixture_paths {
            self.resolve_fixture(fixture_path).await;
        }

        // Apply deferred relation writes. Each relation is looked up in the
        // current state before applying so that channel functions are resolved
        // against the latest fixture definitions.
        let deferred_writes = std::mem::take(&mut self.deferred_relations);
        for (relation, value) in deferred_writes {
            // Look up the target channel function from state.
            let channel_function_opt = {
                let state = self.state.read().await;
                state
                    .patch
                    .fixtures
                    .get(&relation.fixture_path())
                    .and_then(|f| f.channel_function(&relation.attribute()))
                    .cloned()
            };

            if let Some(channel_function) = channel_function_opt {
                self.set_channel_function_value(&channel_function, value).await;
            }
        }
    }

    /// Resolve all channel functions of a single fixture.
    async fn resolve_fixture(&mut self, fixture_path: FixturePath) {
        // Snapshot the fixture's channel functions while holding the state read lock.
        let channel_functions: Vec<(Attribute, FixtureChannelFunction)> = {
            let state = self.state.read().await;
            if let Some(fixture) = state.patch.fixtures.get(&fixture_path) {
                fixture.channel_functions.iter().map(|(a, cf)| (*a, cf.clone())).collect()
            } else {
                Vec::new()
            }
        };

        // For each channel function, get its explicit value (if any) and apply it.
        for (attribute, channel_function) in channel_functions {
            if let Some(value) = self.get_channel_function_value(fixture_path, attribute).await {
                self.set_channel_function_value(&channel_function, value).await;
            }
        }
    }

    /// Determines the value for a specific channel function explicitly present in the GDCS's unresolved values map.
    async fn get_channel_function_value(
        &self,
        fixture_path: FixturePath,
        attribute: Attribute,
    ) -> Option<ClampedValue> {
        let av = self.attribute_values.read().await;
        av.get(fixture_path, attribute)
    }

    /// Apply a computed value to a channel function.
    ///
    /// For physical channel functions, converts the `ClampedValue` to the
    /// appropriate byte sequence and writes it into the multiverse at the
    /// configured addresses.
    ///
    /// For virtual channel functions, evaluates relations and defers the
    /// actual writes so that they can be applied after the initial pass.
    async fn set_channel_function_value(
        &mut self,
        channel_function: &FixtureChannelFunction,
        value: ClampedValue,
    ) {
        match channel_function.kind() {
            FixtureChannelFunctionKind::Physical { addresses } => {
                let values = value.to_address_values(addresses);
                let mut multiverse = self.multiverse.write().await;
                for (address, value) in values {
                    multiverse.set_value(&address, value);
                }
            }
            FixtureChannelFunctionKind::Virtual { relations } => {
                for relation in relations {
                    match *relation.kind() {
                        RelationKind::Multiply => {
                            if let Some(follower_value) = self
                                .get_channel_function_value(
                                    relation.fixture_path(),
                                    relation.attribute(),
                                )
                                .await
                            {
                                let new_value =
                                    ClampedValue::new(follower_value.as_f32() * value.as_f32());
                                self.defer_relation_resolution(relation.clone(), new_value);
                            }
                        }
                        RelationKind::Override => {
                            self.defer_relation_resolution(relation.clone(), value);
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
    fn defer_relation_resolution(&mut self, relation: Relation, value: ClampedValue) {
        self.deferred_relations.push((relation, value));
    }
}
