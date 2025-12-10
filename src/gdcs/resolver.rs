use crate::dmx::{self, Multiverse};
use crate::gdcs::attr::Attribute;
use crate::gdcs::fixture::{
    Fixture, FixtureChannelFunction, FixtureChannelFunctionKind, FixturePath, Relation,
    RelationKind,
};
use crate::gdcs::{ClampedValue, GeneralizedDmxControlSystem};

pub struct Resolver<'gdcs> {
    gdcs: &'gdcs GeneralizedDmxControlSystem,
    multiverse: Multiverse,
    deferred_relations: Vec<(&'gdcs Relation, ClampedValue)>,
}

impl<'gdcs> Resolver<'gdcs> {
    pub fn new(gdcs: &'gdcs GeneralizedDmxControlSystem) -> Self {
        Self { gdcs, multiverse: Multiverse::new(), deferred_relations: Vec::new() }
    }

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

    fn resolve_fixture(&mut self, fixture: &'gdcs Fixture) {
        for (attribute, channel_function) in fixture.channel_functions() {
            let value = self.get_channel_function_value(fixture.path(), attribute);
            self.set_channel_function_value(channel_function, value);
        }
    }

    fn get_channel_function_value(
        &self,
        fixture_path: FixturePath,
        attribute: &Attribute,
    ) -> ClampedValue {
        match self.gdcs.unresolved_values().get(&(fixture_path, attribute.clone())).copied() {
            Some(value) => value,
            None => {
                let Some(fixture) = self.gdcs.fixture(&fixture_path) else {
                    return ClampedValue::default();
                };

                let Some(channel_function) = fixture.channel_function(&attribute) else {
                    return ClampedValue::default();
                };

                channel_function.default()
            }
        }
    }

    fn set_channel_function_value(
        &mut self,
        channel_function: &'gdcs FixtureChannelFunction,
        value: ClampedValue,
    ) {
        match channel_function.kind() {
            FixtureChannelFunctionKind::Physical { addresses } => {
                let bytes: Vec<u8> = match addresses.len() {
                    1 => vec![value.to_u8()],
                    2 => value.to_u16_bytes().to_vec(),
                    3 => value.to_u24_bytes().to_vec(),
                    4 => value.to_u32_bytes().to_vec(),
                    _ => {
                        log::warn!(
                            "cannot set DMX channel value for fixture: unsupported address length {}",
                            addresses.len()
                        );
                        return;
                    }
                };

                for (address, byte) in addresses.iter().zip(bytes) {
                    self.multiverse.set_value(address, dmx::Value(byte));
                }
            }
            FixtureChannelFunctionKind::Virtual { relations } => {
                for relation in relations {
                    match *relation.kind() {
                        RelationKind::Multiply => {
                            let follower_value = self.get_channel_function_value(
                                relation.fixture_path(),
                                relation.attribute(),
                            );

                            let new_value =
                                ClampedValue::new(follower_value.as_f32() * value.as_f32());
                            self.defer_relation_resolution(relation, new_value);
                        }
                        RelationKind::Override => {
                            self.defer_relation_resolution(relation, value);
                        }
                    }
                }
            }
        }
    }

    fn defer_relation_resolution(&mut self, relation: &'gdcs Relation, value: ClampedValue) {
        self.deferred_relations.push((relation, value));
    }
}
