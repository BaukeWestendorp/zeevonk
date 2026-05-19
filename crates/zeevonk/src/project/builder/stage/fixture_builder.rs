use std::collections::HashSet;

use gdtf::dmx_mode::DmxMode;
use gdtf::fixture_type::FixtureType;
use gdtf::geometry::{AnyGeometry, Geometry, ReferenceGeometry};
use gdtf::values::Name;

use crate::project::stage::{Fixture, FixtureId, FixtureIdPart};
use crate::theymx::Address;
use channel_functions::ChannelFunctionCtx;
use virtual_channels::VirtualChannelResolver;

/// Builds a fixture tree from a GDTF fixture type + DMX mode.
pub(crate) struct FixtureBuilder<'a> {
    root_id: FixtureIdPart,
    name: String,
    address: Address,

    gdtf_fixture_type: &'a FixtureType,
    gdtf_dmx_mode: &'a DmxMode,

    fixtures: Vec<Fixture>,
    sibling_count_stack: Vec<u32>,

    channel_ctx: ChannelFunctionCtx<'a>,
    virtuals: VirtualChannelResolver,

    pub(crate) defaults: HashSet<(Address, theymx::Value)>,
}

impl<'a> FixtureBuilder<'a> {
    pub(crate) fn new(
        root_id: FixtureIdPart,
        name: String,
        address: Address,
        gdtf_fixture_type: &'a FixtureType,
        gdtf_dmx_mode: &'a DmxMode,
    ) -> Self {
        Self {
            root_id,
            name,
            address,
            gdtf_fixture_type,
            gdtf_dmx_mode,
            fixtures: Vec::new(),
            sibling_count_stack: Vec::new(),
            channel_ctx: ChannelFunctionCtx::new(gdtf_fixture_type, gdtf_dmx_mode),
            virtuals: VirtualChannelResolver::new(),
            defaults: HashSet::new(),
        }
    }

    pub(crate) fn build_fixture_tree(
        mut self,
    ) -> crate::Result<(Vec<Fixture>, HashSet<(Address, theymx::Value)>)> {
        let root_geometry = self.root_geometry()?.clone();
        let root_id = FixtureId::new(self.root_id);

        self.fixtures = self.fixtures_from_geometry(root_id, &root_geometry);

        self.virtuals.resolve_all(
            self.gdtf_dmx_mode,
            self.gdtf_fixture_type,
            &self.channel_ctx,
            &mut self.fixtures,
        );

        Ok((self.fixtures, self.defaults))
    }

    fn root_geometry(&self) -> crate::Result<&Geometry> {
        self.gdtf_dmx_mode
            .geometry(self.gdtf_fixture_type)
            .ok_or(crate::Error::RootGeometryNotFound)
    }

    fn fixtures_from_geometry(&mut self, child_id: FixtureId, geometry: &Geometry) -> Vec<Fixture> {
        self.sibling_count_stack.push(0);

        let fixtures = match geometry {
            Geometry::Reference(reference) => {
                self.fixture_from_reference_geometry(child_id, reference)
            }
            geom => self.fixture_from_geometry(child_id, geom),
        };

        self.sibling_count_stack.pop();

        fixtures
    }

    fn fixture_from_geometry(&mut self, child_id: FixtureId, geometry: &Geometry) -> Vec<Fixture> {
        let name = if child_id.len() == 1 {
            self.name.clone()
        } else {
            geometry.name().map(|n| n.to_string()).unwrap_or_else(|| "<no name>".to_string())
        };

        let geometry_name = match geometry.name() {
            Some(n) => n,
            None => return vec![],
        };

        self.create_child_fixture(child_id, name, geometry_name, geometry_name, 0)
    }

    fn fixture_from_reference_geometry(
        &mut self,
        child_id: FixtureId,
        reference_geometry: &ReferenceGeometry,
    ) -> Vec<Fixture> {
        if reference_geometry.breaks.len() > 1 {
            log::warn!("multiple breaks not yet supported!");
        }

        let geometry_address_offset = match reference_geometry.breaks.first() {
            Some(b) => b.dmx_offset.absolute() as i32 - 1,
            None => 0,
        };

        let geometry_name = match reference_geometry.name() {
            Some(n) => n,
            None => return vec![],
        };
        let referenced_geometry_name = match reference_geometry.geometry.as_ref() {
            Some(n) => n,
            None => return vec![],
        };

        self.create_child_fixture(
            child_id,
            geometry_name.to_string(),
            geometry_name,
            referenced_geometry_name,
            geometry_address_offset,
        )
    }

    fn create_child_fixture(
        &mut self,
        id: FixtureId,
        name: String,
        geometry: &Name,
        referenced_geometry: &Name,
        geometry_address_offset: i32,
    ) -> Vec<Fixture> {
        let Some(referenced_geometry) = self.gdtf_fixture_type.nested_geometry(referenced_geometry)
        else {
            log::error!(
                "Referenced geometry {:?} not found in fixture type {:?}",
                referenced_geometry,
                self.gdtf_fixture_type.fixture_type_id
            );
            return vec![];
        };

        let child_fixtures = self.collect_child_fixtures(&id, referenced_geometry);
        let child_ids = self.collect_direct_child_ids(&id, &child_fixtures);

        // Keep unwrap to preserve prior behavior.
        let referenced_name = referenced_geometry.name().unwrap();

        let gdtf_dmx_mode_name = self
            .gdtf_dmx_mode
            .name
            .as_ref()
            .map(|n| n.to_string())
            .unwrap_or_else(|| "<no mode name>".to_string());

        let (channel_functions, highlight_values) = self.channel_ctx.create_channel_functions(
            id,
            geometry,
            referenced_name,
            geometry_address_offset,
            self.address,
            &mut self.defaults,
            &mut self.virtuals,
        );

        let mut fixtures = vec![Fixture {
            id,
            root_base_address: self.address,
            name,
            gdtf_fixture_type_id: self.gdtf_fixture_type.fixture_type_id,
            gdtf_dmx_mode: gdtf_dmx_mode_name,
            channel_functions,
            highlight_values,
            child_ids,
        }];

        fixtures.extend(child_fixtures);
        fixtures
    }

    fn collect_child_fixtures(&mut self, id: &FixtureId, geometry: &Geometry) -> Vec<Fixture> {
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

            let fixtures_for_child = self.fixtures_from_geometry(part, child_geometry);
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
    use std::collections::{BTreeMap, HashSet};
    use std::str::FromStr;

    use gdtf::dmx_mode::{ChannelFunction, DmxChannel, DmxMode};
    use gdtf::fixture_type::FixtureType;
    use gdtf::values::Name;
    use rigger::gdtf::attr::AttributeName;

    use crate::project::stage::{FixtureChannelFunction, FixtureChannelFunctionKind, FixtureId};
    use crate::theymx::Address;
    use crate::value::ClampedValue;

    use super::virtual_channels::VirtualChannelResolver;

    /// Context for creating channel functions for a fixture instance.
    ///
    /// This owns the static GDTF inputs needed to interpret channel functions.
    pub(crate) struct ChannelFunctionCtx<'a> {
        gdtf_fixture_type: &'a FixtureType,
        gdtf_dmx_mode: &'a DmxMode,
    }

    impl<'a> ChannelFunctionCtx<'a> {
        pub(crate) fn new(gdtf_fixture_type: &'a FixtureType, gdtf_dmx_mode: &'a DmxMode) -> Self {
            Self { gdtf_fixture_type, gdtf_dmx_mode }
        }

        pub(crate) fn create_channel_functions(
            &self,
            id: FixtureId,
            geometry: &Name,
            referenced_geometry: &Name,
            geometry_address_offset: i32,
            base_address: Address,
            defaults: &mut HashSet<(Address, theymx::Value)>,
            virtuals: &mut VirtualChannelResolver,
        ) -> (
            BTreeMap<AttributeName, FixtureChannelFunction>,
            BTreeMap<Address, crate::theymx::Value>,
        ) {
            let dmx_channels_with_geometry = self
                .gdtf_dmx_mode
                .dmx_channels
                .iter()
                .enumerate()
                .filter(|(_, dmx_channel)| dmx_channel.geometry == *referenced_geometry);

            let mut channel_functions = BTreeMap::new();

            let mut highlight_values = BTreeMap::new();

            for (c_ix, dmx_channel) in dmx_channels_with_geometry {
                for (lc_ix, logical_channel) in dmx_channel.logical_channels.iter().enumerate() {
                    let filtered_channel_functions = logical_channel
                        .channel_functions
                        .iter()
                        .filter(|cf| {
                            cf.attribute(self.gdtf_fixture_type).is_some_and(|a| {
                                a.name.as_ref().is_some_and(|name| &**name != "NoFeature")
                            })
                        })
                        .enumerate()
                        .collect::<Vec<_>>();

                    for (cf_ix, channel_function) in &filtered_channel_functions {
                        let from = channel_function.physical_from as f32;
                        let to = channel_function.physical_to as f32;

                        // Map the clamped value to the physical range [from, to]
                        let default_clamped = ClampedValue::from(channel_function.default);
                        let default = from + (to - from) * default_clamped.as_f32();

                        let Some(attribute) = self.attribute_from_cf(channel_function) else {
                            continue;
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
                            attribute.clone(),
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
                                let default_values = default_clamped.to_address_values(addresses);
                                defaults.extend(default_values);

                                if let Some(highlight) = dmx_channel.highlight {
                                    let values =
                                        ClampedValue::from(highlight).to_address_values(addresses);
                                    for (address, value) in values {
                                        highlight_values.insert(address, value);
                                    }
                                }
                            }
                        }

                        channel_functions.insert(
                            attribute,
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
            match &dmx_channel.offset {
                Some(offsets) => {
                    let addresses = offsets
                        .iter()
                        .filter_map(|o| {
                            match base_address.with_channel_offset(geometry_address_offset + o - 1)
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
                None => {
                    virtuals.register_virtual_channel(cf_id, attribute);
                    FixtureChannelFunctionKind::Virtual { relations: vec![] }
                }
            }
        }

        fn attribute_from_cf(&self, cf: &ChannelFunction) -> Option<AttributeName> {
            cf.attribute(self.gdtf_fixture_type)
                .and_then(|attribute| attribute.name.as_ref())
                .map(|attribute| AttributeName::from_str(attribute).unwrap())
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

    use gdtf::dmx_mode::{ChannelFunction, DmxChannel, DmxMode, RelationType};
    use gdtf::fixture_type::FixtureType;
    use rigger::gdtf::attr::AttributeName;

    use crate::project::stage::{
        Fixture, FixtureChannelFunctionKind, FixtureId, Relation, RelationKind,
    };

    use super::channel_functions::ChannelFunctionId;

    /// Collects and resolves virtual channel functions.
    ///
    /// During channel-function creation we don't yet have enough information to populate
    /// relations for virtual channels. We therefore:
    /// - record which channel function was created in which fixture (`record_channel_function_location`)
    /// - register which channel function is virtual and needs relations (`register_virtual_channel`)
    /// - resolve all virtuals after the fixture list is fully built (`resolve_all`)
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
            gdtf_fixture_type: &FixtureType,
            channel_ctx: &super::channel_functions::ChannelFunctionCtx<'_>,
            fixtures: &mut [Fixture],
        ) {
            // `channel_ctx` is currently only used for attribute parsing semantics; keep it in the
            // signature to avoid re-threading later when this logic is further split/purified.
            let _ = channel_ctx;

            for (cf_id, virtual_attribute) in &self.unresolved_virtual_channels {
                let Some(dmx_channel) = gdtf_dmx_mode.dmx_channels.get(cf_id.channel_ix) else {
                    continue;
                };

                let relations = self.relations_for_dmx_channel(
                    gdtf_dmx_mode,
                    gdtf_fixture_type,
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

                virtual_channel_function.kind = FixtureChannelFunctionKind::Virtual { relations };
            }
        }

        fn relations_for_dmx_channel(
            &self,
            gdtf_dmx_mode: &DmxMode,
            gdtf_fixture_type: &FixtureType,
            geometry: &str,
            dmx_channel: &DmxChannel,
        ) -> Vec<Relation> {
            let mut channel_relations = Vec::new();

            let relations = gdtf_dmx_mode.relations.iter().filter(|relation| {
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

                let kind = match relation.type_ {
                    RelationType::Multiply => RelationKind::Multiply,
                    RelationType::Override => RelationKind::Override,
                };

                let Some(fixture_id) = self.fixture_id_for_channel_function(
                    gdtf_dmx_mode,
                    geometry,
                    follower_channel_function,
                ) else {
                    log::warn!(
                        "could not find fixture id for follower channel function {}",
                        follower_channel_function.name.as_deref().unwrap_or("<no name>")
                    );
                    continue;
                };

                let Some(attribute) =
                    attribute_from_cf(gdtf_fixture_type, follower_channel_function)
                else {
                    continue;
                };

                channel_relations.push(Relation::new(kind, fixture_id, attribute));
            }

            channel_relations
        }

        fn fixture_id_for_channel_function(
            &self,
            gdtf_dmx_mode: &DmxMode,
            geometry: &str,
            target_channel_function: &ChannelFunction,
        ) -> Option<FixtureId> {
            for (c_ix, dmx_channel) in gdtf_dmx_mode.dmx_channels.iter().enumerate() {
                for (lc_ix, logical_channel) in dmx_channel.logical_channels.iter().enumerate() {
                    for (cf_ix, channel_function) in
                        logical_channel.channel_functions.iter().enumerate()
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

    fn attribute_from_cf(
        gdtf_fixture_type: &FixtureType,
        cf: &ChannelFunction,
    ) -> Option<AttributeName> {
        use std::str::FromStr;

        cf.attribute(gdtf_fixture_type)
            .and_then(|attribute| attribute.name.as_ref())
            .map(|attribute| AttributeName::from_str(attribute).unwrap())
    }
}
