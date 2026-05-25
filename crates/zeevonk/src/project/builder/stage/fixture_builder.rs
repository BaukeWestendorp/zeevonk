use crate::value::AttributeValues;

use rigger::gdtf::Gdtf;
use rigger::gdtf::Name;
use rigger::gdtf::dmx::DmxMode;
use rigger::gdtf::geo::{Geometry, ReferenceGeometry};

use crate::project::stage::{Fixture, FixtureId, FixtureIdPart};
use crate::theymx::Address;
use channel_functions::ChannelFunctionContext;
use virtual_channels::VirtualChannelResolver;

/// Builds a fixture tree from a GDTF fixture type + DMX mode.
pub(crate) struct FixtureBuilder<'a> {
    root_id: FixtureIdPart,
    name: String,
    address: Address,

    gdtf: &'a Gdtf,
    gdtf_dmx_mode: &'a DmxMode,

    fixtures: Vec<Fixture>,
    sibling_count_stack: Vec<u32>,

    channel_cx: ChannelFunctionContext<'a>,
    virtuals: VirtualChannelResolver,

    pub(crate) defaults: AttributeValues,
}

impl<'a> FixtureBuilder<'a> {
    pub(crate) fn new(
        root_id: FixtureIdPart,
        name: String,
        address: Address,
        gdtf: &'a Gdtf,
        gdtf_dmx_mode: &'a DmxMode,
    ) -> Self {
        Self {
            root_id,
            name,
            address,
            gdtf,
            gdtf_dmx_mode,
            fixtures: Vec::new(),
            sibling_count_stack: Vec::new(),
            channel_cx: ChannelFunctionContext::new(gdtf, gdtf_dmx_mode),
            virtuals: VirtualChannelResolver::new(),
            defaults: AttributeValues::new(),
        }
    }

    pub(crate) fn build_fixture_tree(mut self) -> crate::Result<(Vec<Fixture>, AttributeValues)> {
        let root_geometry = self.root_geometry()?.clone();
        let root_id = FixtureId::new(self.root_id);

        self.fixtures = self.fixtures_from_geometry(root_id, &root_geometry, self.gdtf);

        self.virtuals.resolve_all(
            self.gdtf_dmx_mode,
            self.gdtf,
            &self.channel_cx,
            &mut self.fixtures,
            &mut self.defaults,
        );

        Ok((self.fixtures, self.defaults))
    }

    fn root_geometry(&self) -> crate::Result<&Geometry> {
        self.gdtf_dmx_mode.geometry(self.gdtf).ok_or(crate::Error::RootGeometryNotFound)
    }

    fn fixtures_from_geometry(
        &mut self,
        child_id: FixtureId,
        geometry: &Geometry,
        gdtf: &'a Gdtf,
    ) -> Vec<Fixture> {
        self.sibling_count_stack.push(0);

        let fixtures = match geometry {
            Geometry::GeometryReference(reference) => {
                self.fixture_from_reference_geometry(child_id, reference, gdtf)
            }
            geo => self.fixture_from_geometry(child_id, geo, gdtf),
        };

        self.sibling_count_stack.pop();

        fixtures
    }

    fn fixture_from_geometry(
        &mut self,
        child_id: FixtureId,
        geometry: &Geometry,
        gdtf: &'a Gdtf,
    ) -> Vec<Fixture> {
        let name =
            if child_id.len() == 1 { self.name.clone() } else { geometry.name().to_string() };

        self.create_child_fixture(child_id, name, geometry.name(), geometry.name(), 0, gdtf)
    }

    fn fixture_from_reference_geometry(
        &mut self,
        child_id: FixtureId,
        reference_geometry: &ReferenceGeometry,
        gdtf: &'a Gdtf,
    ) -> Vec<Fixture> {
        if reference_geometry.breaks().len() > 1 {
            log::warn!("multiple breaks not yet supported!");
        }

        let geometry_address_offset = match reference_geometry.breaks().first() {
            Some(b) => b.absolute() as i32 - 1,
            None => 0,
        };

        let geometry_name = reference_geometry.name();

        let referenced_geometry_name = match reference_geometry.geometry(gdtf).as_ref() {
            Some(n) => n.name(),
            None => return vec![],
        };

        self.create_child_fixture(
            child_id,
            geometry_name.to_string(),
            geometry_name,
            referenced_geometry_name,
            geometry_address_offset,
            gdtf,
        )
    }

    fn create_child_fixture(
        &mut self,
        id: FixtureId,
        name: String,
        geometry: &Name,
        referenced_geometry: &Name,
        geometry_address_offset: i32,
        gdtf: &'a Gdtf,
    ) -> Vec<Fixture> {
        let Some(referenced_geometry) = self.gdtf.geometry(referenced_geometry) else {
            log::error!(
                "Referenced geometry '{}' not found in fixture type '{}'",
                referenced_geometry,
                self.gdtf.name()
            );
            return vec![];
        };

        let child_fixtures = self.collect_child_fixtures(&id, referenced_geometry, gdtf);
        let child_ids = self.collect_direct_child_ids(&id, &child_fixtures);

        let referenced_name = referenced_geometry.name();

        let gdtf_dmx_mode = self.gdtf_dmx_mode.name().to_string();

        let (channel_functions, highlight_values) = self.channel_cx.create_channel_functions(
            id,
            geometry,
            referenced_name,
            geometry_address_offset,
            self.address,
            &mut self.defaults,
            &mut self.virtuals,
            gdtf,
        );

        let mut fixtures = vec![Fixture {
            id,
            root_base_address: self.address,
            name,
            gdtf_fixture_type_id: self.gdtf.fixture_type_id(),
            gdtf_dmx_mode,
            channel_functions,
            highlight_values,
            child_ids,
        }];

        fixtures.extend(child_fixtures);
        fixtures
    }

    fn collect_child_fixtures(
        &mut self,
        id: &FixtureId,
        geometry: &Geometry,
        gdtf: &'a Gdtf,
    ) -> Vec<Fixture> {
        let mut child_fixtures = Vec::new();

        for child_geometry in geometry.children() {
            let sibling_count = *self.sibling_count_stack.last().unwrap();

            let part = match FixtureIdPart::new(sibling_count + 1) {
                Ok(part) => id.extended_with(part),
                Err(err) => {
                    log::error!("invalid FixtureIdPart: {}", err);
                    continue;
                }
            };

            let fixtures_for_child = self.fixtures_from_geometry(part, child_geometry, gdtf);
            if fixtures_for_child.is_empty() {
                continue;
            }

            let parent_fixture = &fixtures_for_child[0];
            if parent_fixture.channel_functions.is_empty() && parent_fixture.child_ids.is_empty() {
                continue;
            }

            *self.sibling_count_stack.last_mut().unwrap() += 1;
            child_fixtures.extend(fixtures_for_child);
        }

        child_fixtures
    }

    fn collect_direct_child_ids(
        &self,
        id: &FixtureId,
        child_fixtures: &[Fixture],
    ) -> Vec<FixtureId> {
        child_fixtures
            .iter()
            .map(|f| f.id())
            .filter(|child_id| child_id.len() == id.len() + 1)
            .collect()
    }
}

mod channel_functions {
    use std::collections::BTreeMap;

    use rigger::gdtf::attr::{AttributeName, PhysicalUnit};
    use rigger::gdtf::dmx::{DmxChannel, DmxMode, DmxOffset};
    use rigger::gdtf::{Gdtf, Name};

    use crate::project::stage::{FixtureChannelFunction, FixtureChannelFunctionKind, FixtureId};
    use crate::theymx::Address;
    use crate::value::{AttributeValue, AttributeValues, ClampedValue};

    use super::virtual_channels::VirtualChannelResolver;

    /// Context for creating channel functions for a fixture instance.
    ///
    /// This owns the static GDTF inputs needed to interpret channel functions.
    pub(crate) struct ChannelFunctionContext<'a> {
        gdtf: &'a Gdtf,
        gdtf_dmx_mode: &'a DmxMode,
    }

    impl<'a> ChannelFunctionContext<'a> {
        pub(crate) fn new(gdtf: &'a Gdtf, gdtf_dmx_mode: &'a DmxMode) -> Self {
            Self { gdtf, gdtf_dmx_mode }
        }

        pub(crate) fn create_channel_functions(
            &self,
            id: FixtureId,
            geometry: &Name,
            referenced_geometry: &Name,
            geometry_address_offset: i32,
            base_address: Address,
            defaults: &mut AttributeValues,
            virtuals: &mut VirtualChannelResolver,
            gdtf: &'a Gdtf,
        ) -> (
            BTreeMap<AttributeName, FixtureChannelFunction>,
            BTreeMap<Address, crate::theymx::Value>,
        ) {
            let dmx_channels_with_geometry =
                self.gdtf_dmx_mode.dmx_channels().iter().enumerate().filter(|(_, dmx_channel)| {
                    dmx_channel.geometry(gdtf).is_some_and(|geo| geo.name() == referenced_geometry)
                });

            let mut channel_functions = BTreeMap::new();

            let mut highlight_values = BTreeMap::new();

            for (c_ix, dmx_channel) in dmx_channels_with_geometry {
                for (lc_ix, logical_channel) in dmx_channel.logical_channels().iter().enumerate() {
                    let filtered_channel_functions = logical_channel
                        .channel_functions()
                        .iter()
                        .filter(|cf| {
                            cf.attribute(self.gdtf)
                                .is_some_and(|a| a.name() != &AttributeName::NoFeature)
                        })
                        .enumerate()
                        .collect::<Vec<_>>();

                    for (cf_ix, channel_function) in &filtered_channel_functions {
                        let Some(attribute) = channel_function.attribute(gdtf) else {
                            log::error!("attribute not found for channel function");
                            continue;
                        };

                        let is_unitless = matches!(attribute.physical_unit(), PhysicalUnit::None);

                        let default_clamped = ClampedValue::from(channel_function.default());

                        let (from, to, default) = if is_unitless {
                            (
                                AttributeValue::Clamped(ClampedValue::new(0.0)),
                                AttributeValue::Clamped(ClampedValue::new(1.0)),
                                AttributeValue::Clamped(default_clamped),
                            )
                        } else {
                            let from = channel_function.physical_from();
                            let to = channel_function.physical_to();
                            let default = from + (to - from) * default_clamped.as_f32();

                            (
                                AttributeValue::Physical(from),
                                AttributeValue::Physical(to),
                                AttributeValue::Physical(default),
                            )
                        };

                        let cf_id = ChannelFunctionId {
                            fixture_id: id,
                            geometry: geometry.to_string(),
                            channel_ix: c_ix,
                            logical_channel_ix: lc_ix,
                            channel_function_ix: *cf_ix,
                        };

                        let kind = self.make_channel_function_kind(
                            dmx_channel,
                            attribute.name().clone(),
                            cf_id.clone(),
                            geometry_address_offset,
                            base_address,
                            virtuals,
                        );

                        if dmx_channel
                            .initial_function()
                            .is_some_and(|(_, cf)| cf == *channel_function)
                        {
                            if let FixtureChannelFunctionKind::Physical { addresses } = &kind {
                                defaults.set(id, attribute.name().clone(), default);

                                if let Some(highlight) = dmx_channel.highlight() {
                                    let values =
                                        ClampedValue::from(highlight).to_address_values(addresses);
                                    for (address, value) in values {
                                        highlight_values.insert(address, value);
                                    }
                                }
                            }
                        }

                        channel_functions.insert(
                            attribute.name().clone(),
                            FixtureChannelFunction { kind, min: from, max: to, default },
                        );

                        virtuals.record_channel_function_location(cf_id, id);
                    }
                }
            }

            (channel_functions, highlight_values)
        }

        fn make_channel_function_kind(
            &self,
            dmx_channel: &DmxChannel,
            attribute: AttributeName,
            cf_id: ChannelFunctionId,
            geometry_address_offset: i32,
            base_address: Address,
            virtuals: &mut VirtualChannelResolver,
        ) -> FixtureChannelFunctionKind {
            match &dmx_channel.offset() {
                DmxOffset::Physical(offsets) => {
                    let addresses = offsets
                        .iter()
                        .filter_map(|o| {
                            match base_address
                                .with_channel_offset(geometry_address_offset + *o as i32 - 1)
                            {
                                Ok(addr) => Some(addr),
                                Err(err) => {
                                    log::error!("failed to compute channel offset: {}", err);
                                    None
                                }
                            }
                        })
                        .collect();

                    FixtureChannelFunctionKind::Physical { addresses }
                }
                DmxOffset::Virtual => {
                    virtuals.register_virtual_channel(cf_id, attribute);
                    FixtureChannelFunctionKind::Virtual { relations: vec![] }
                }
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub(crate) struct ChannelFunctionId {
        pub(crate) fixture_id: FixtureId,
        pub(crate) geometry: String,
        pub(crate) channel_ix: usize,
        pub(crate) logical_channel_ix: usize,
        pub(crate) channel_function_ix: usize,
    }
}

mod virtual_channels {
    use std::collections::BTreeMap;

    use rigger::gdtf::Gdtf;
    use rigger::gdtf::attr::AttributeName;
    use rigger::gdtf::dmx::{
        ChannelFunction, DmxChannel, DmxMode, RelationKind as RiggerRelationKind,
    };

    use crate::project::stage::{
        Fixture, FixtureChannelFunctionKind, FixtureId, Relation, RelationKind,
    };
    use crate::value::{AttributeValues, ClampedValue};

    use super::channel_functions::{ChannelFunctionContext, ChannelFunctionId};

    /// Collects and resolves virtual channel functions.
    ///
    /// During channel-function creation we don't yet have enough information to populate
    /// relations for virtual channels.
    pub(crate) struct VirtualChannelResolver {
        channel_function_map: BTreeMap<ChannelFunctionId, FixtureId>,
        unresolved_virtual_channels: Vec<(ChannelFunctionId, AttributeName)>,
    }

    impl VirtualChannelResolver {
        pub(crate) fn new() -> Self {
            Self { channel_function_map: BTreeMap::new(), unresolved_virtual_channels: Vec::new() }
        }

        pub(crate) fn record_channel_function_location(
            &mut self,
            cf_id: ChannelFunctionId,
            id: FixtureId,
        ) {
            self.channel_function_map.insert(cf_id, id);
        }

        pub(crate) fn register_virtual_channel(
            &mut self,
            cf_id: ChannelFunctionId,
            attribute: AttributeName,
        ) {
            self.unresolved_virtual_channels.push((cf_id, attribute));
        }

        pub(crate) fn resolve_all(
            &self,
            gdtf_dmx_mode: &DmxMode,
            gdtf: &Gdtf,
            channel_cx: &ChannelFunctionContext<'_>,
            fixtures: &mut [Fixture],
            defaults: &mut AttributeValues,
        ) {
            // `channel_cx` is currently only used for attribute parsing semantics; keep it in the
            // signature to avoid re-threading later when this logic is further split/purified.
            let _ = channel_cx;

            for (cf_id, virtual_attribute) in &self.unresolved_virtual_channels {
                let Some(dmx_channel) = gdtf_dmx_mode.dmx_channels().get(cf_id.channel_ix) else {
                    continue;
                };

                let Some(logical_channel) =
                    dmx_channel.logical_channels().get(cf_id.logical_channel_ix)
                else {
                    continue;
                };

                let Some(channel_function) =
                    logical_channel.channel_functions().get(cf_id.channel_function_ix)
                else {
                    continue;
                };

                let relations = self.relations_for_dmx_channel(
                    gdtf_dmx_mode,
                    gdtf,
                    &cf_id.geometry,
                    dmx_channel,
                );

                let Some(fixture) = fixtures.iter_mut().find(|f| f.id() == cf_id.fixture_id) else {
                    continue;
                };

                let Some(virtual_channel_function) =
                    fixture.channel_functions.get_mut(virtual_attribute)
                else {
                    continue;
                };

                virtual_channel_function.kind =
                    FixtureChannelFunctionKind::Virtual { relations: relations.clone() };

                let is_initial = dmx_channel
                    .initial_function()
                    .is_some_and(|(_, cf)| std::ptr::eq(cf, channel_function));

                if is_initial {
                    defaults.set(
                        cf_id.fixture_id,
                        virtual_attribute.clone(),
                        crate::value::AttributeValue::Clamped(ClampedValue::from(
                            channel_function.default(),
                        )),
                    );
                }
            }
        }

        fn relations_for_dmx_channel(
            &self,
            gdtf_dmx_mode: &DmxMode,
            gdtf: &Gdtf,
            geometry: &str,
            dmx_channel: &DmxChannel,
        ) -> Vec<Relation> {
            let mut channel_relations = Vec::new();

            let relations = gdtf_dmx_mode.relations().iter().filter(|relation| {
                relation
                    .master(gdtf_dmx_mode)
                    .is_some_and(|master| master.name() == dmx_channel.name())
            });

            for relation in relations {
                let Some((_, _, follower_channel_function)) = relation.follower(gdtf_dmx_mode)
                else {
                    log::warn!(
                        "could not find follower for relation with master {}",
                        dmx_channel.name()
                    );
                    continue;
                };

                let kind = match relation.kind() {
                    RiggerRelationKind::Multiply => RelationKind::Multiply,
                    RiggerRelationKind::Override => RelationKind::Override,
                };

                let Some(fixture_id) = self.fixture_id_for_channel_function(
                    gdtf_dmx_mode,
                    geometry,
                    follower_channel_function,
                ) else {
                    log::warn!(
                        "could not find fixture id for follower channel function {}",
                        follower_channel_function
                            .name()
                            .map(|cf| cf.as_str())
                            .unwrap_or("<no name>")
                    );
                    continue;
                };

                let Some(attribute) = follower_channel_function.attribute(gdtf) else {
                    continue;
                };

                channel_relations.push(Relation::new(kind, fixture_id, attribute.name().clone()));
            }

            channel_relations
        }

        fn fixture_id_for_channel_function(
            &self,
            gdtf_dmx_mode: &DmxMode,
            geometry: &str,
            target_channel_function: &ChannelFunction,
        ) -> Option<FixtureId> {
            for (c_ix, dmx_channel) in gdtf_dmx_mode.dmx_channels().iter().enumerate() {
                for (lc_ix, logical_channel) in dmx_channel.logical_channels().iter().enumerate() {
                    for (cf_ix, channel_function) in
                        logical_channel.channel_functions().iter().enumerate()
                    {
                        if !std::ptr::eq(target_channel_function, channel_function) {
                            continue;
                        }

                        if let Some((_, fixture_id)) =
                            self.channel_function_map.iter().find(|(id, _)| {
                                id.geometry == geometry
                                    && id.channel_ix == c_ix
                                    && id.logical_channel_ix == lc_ix
                                    && id.channel_function_ix == cf_ix
                            })
                        {
                            return Some(*fixture_id);
                        }
                    }
                }
            }

            None
        }
    }
}
