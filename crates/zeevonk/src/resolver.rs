use theymx::Multiverse;

use crate::attr::Attribute;
use crate::project::stage::{
    FixtureChannelFunction, FixtureChannelFunctionKind, FixtureId, Relation, RelationKind, Stage,
};
use crate::value::{AttributeValues, ClampedValue};

pub fn resolve(values: &AttributeValues, stage: &Stage, multiverse: &mut Multiverse) {
    Resolver::new(values, stage, multiverse).resolve();
}

/// Resolver for translating Zeevonk state into a physical DMX multiverse.
///
/// The resolver walks the fixtures, computes the effective value for
/// each fixture channel function, and writes the corresponding bytes into a
/// [`theymx::Multiverse`]. Virtual channel functions (those driven by relations)
/// are resolved by deferring relation writes until all fixtures have been
/// examined. This allows follower relations (multiply or override) to be
/// resolved against the master's computed values.
struct Resolver<'a> {
    attribute_values: &'a AttributeValues,
    stage: &'a Stage,
    multiverse: &'a mut Multiverse,

    /// Relations whose writes are deferred until after the initial fixture
    /// pass. Each entry contains the relation and the resolved value to apply.
    /// This is needed for resolving virtual channels.
    deferred_relations: Vec<(Relation, ClampedValue)>,
}

impl<'a> Resolver<'a> {
    /// Create a new resolver.
    pub fn new(
        attribute_values: &'a AttributeValues,
        stage: &'a Stage,
        multiverse: &'a mut Multiverse,
    ) -> Self {
        Self { attribute_values, stage, multiverse, deferred_relations: Vec::new() }
    }

    /// Perform resolution and return the populated multiverse.
    pub fn resolve(mut self) {
        let fixture_ids: Vec<FixtureId> = { self.stage.fixtures().keys().cloned().collect() };

        // Resolve each fixture independently.
        for fixture_id in fixture_ids {
            self.resolve_fixture(fixture_id);
        }

        // FIXME: This goes only one layer of deferring deep. It might be possible to have two or more
        // FIXME: layers of virtual channel chaining, but only the first layer gets deferred.
        // Apply deferred relation writes. Each relation is looked up in the
        // current show data before applying so that channel functions are resolved
        // against the latest fixture definitions.
        let deferred_writes = std::mem::take(&mut self.deferred_relations);
        for (relation, value) in deferred_writes {
            // Look up the target channel function from show data.
            let channel_function_opt = {
                self.stage
                    .fixtures()
                    .get(&relation.fixture_id())
                    .and_then(|f| f.channel_function(&relation.attribute()))
                    .cloned()
            };

            if let Some(channel_function) = channel_function_opt {
                self.set_channel_function_value(&channel_function, value);
            }
        }
    }

    /// Resolve all channel functions of a single fixture.
    fn resolve_fixture(&mut self, fixture_id: FixtureId) {
        let Some(fixture) = self.stage.fixtures().get(&fixture_id) else {
            return;
        };

        // For each channel function, get its explicit value (if any) and apply it.
        for (attribute, channel_function) in fixture.channel_functions() {
            if let Some(value) = self.get_channel_function_value(&fixture_id, &attribute) {
                self.set_channel_function_value(channel_function, value);
            }
        }
    }

    /// Determines the value for a specific channel function explicitly present in the Zeevonk's unresolved values map.
    fn get_channel_function_value(
        &self,
        fixture_id: &FixtureId,
        attribute: &Attribute,
    ) -> Option<ClampedValue> {
        self.attribute_values.get(fixture_id, attribute)
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
        channel_function: &FixtureChannelFunction,
        value: ClampedValue,
    ) {
        match channel_function.kind() {
            FixtureChannelFunctionKind::Physical { addresses } => {
                let values = value.to_address_values(addresses);
                for (address, value) in values {
                    self.multiverse.set_value(&address, value);
                }
            }
            FixtureChannelFunctionKind::Virtual { relations } => {
                for relation in relations {
                    match *relation.kind() {
                        RelationKind::Multiply => {
                            if let Some(follower_value) = self.get_channel_function_value(
                                &relation.fixture_id(),
                                &relation.attribute(),
                            ) {
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
