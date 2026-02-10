use crate::project::{Project, file::ProjectFile};

/// Converts a [`ProjectFile`] to a [`Project`].
pub fn from_file(file: ProjectFile) -> crate::Result<Project> {
    let stage = stage::from_file(&file)?;

    Ok(Project { file, stage })
}

mod stage {
    use std::collections::{HashMap, HashSet};
    use std::fs;
    use std::str::FromStr;

    use crate::theymx::{Address, Multiverse};
    use gdtf::dmx_mode::{ChannelFunction, DmxChannel, DmxMode, RelationType};
    use gdtf::fixture_type::FixtureType;
    use gdtf::geometry::{AnyGeometry, Geometry, ReferenceGeometry};
    use gdtf::values::Name;

    use crate::attr::Attribute;
    use crate::project::file::ProjectFile;
    use crate::project::stage::{
        Fixture, FixtureChannelFunction, FixtureChannelFunctionKind, FixtureId, FixtureIdPart,
        Relation, RelationKind, Stage,
    };
    use crate::value::ClampedValue;

    pub fn from_file(file: &ProjectFile) -> crate::Result<Stage> {
        let mut stage = Stage { fixtures: HashMap::new(), defaulted_multiverse: Multiverse::new() };

        // Get all fixture types used in the project stage.
        let mut fixture_types = HashMap::new();
        for gdtf_file_path in &file.patch.gdtf_file_paths {
            let file = fs::File::open(gdtf_file_path).map_err(|err| {
                std::io::Error::other(format!(
                    "Failed to open GDTF file '{}': {}",
                    gdtf_file_path.display(),
                    err
                ))
            })?;
            let gdtf_file = gdtf::GdtfFile::new(file).map_err(|err| {
                std::io::Error::other(format!(
                    "Failed to parse GDTF file '{}': {}",
                    gdtf_file_path.display(),
                    err
                ))
            })?;

            for fixture_type in gdtf_file.description.fixture_types {
                let fixture_type_id = fixture_type.fixture_type_id;
                fixture_types.insert(fixture_type_id, fixture_type);
            }
        }

        // Build all fixtures in in the project.
        for fixture in &file.patch.fixtures {
            let fixture_type =
                fixture_types.get(&fixture.kind.gdtf_fixture_type_id).ok_or_else(|| {
                    crate::Error::FixtureTypeNotFound { id: fixture.kind.gdtf_fixture_type_id }
                })?;

            let dmx_mode = fixture_type.dmx_mode(&fixture.kind.gdtf_dmx_mode).ok_or_else(|| {
                crate::Error::DmxModeNotFound {
                    mode: fixture.kind.gdtf_dmx_mode.to_string(),
                    fixture_type_id: fixture.kind.gdtf_fixture_type_id,
                }
            })?;

            let builder = FixtureBuilder::new(
                fixture.root_id,
                fixture.name.to_owned(),
                fixture.address,
                fixture_type,
                dmx_mode,
            );

            let (built_fixtures, defaults) = builder.build_fixture_tree()?;
            for built_fixture in built_fixtures {
                stage.fixtures.insert(built_fixture.id(), built_fixture);
            }
            for (address, value) in defaults {
                stage.defaulted_multiverse.set_value(&address, value);
            }
        }

        Ok(stage)
    }

    /// Helper for building the fixture tree from a GDTF fixture type + DMX mode.
    ///
    /// The builder walks the nested geometry tree, constructs fixtures and their channel
    /// functions (physical or virtual), and resolves relations for virtual channels after the
    /// first pass.
    struct FixtureBuilder<'a> {
        root_id: FixtureIdPart,
        name: String,
        address: Address,

        gdtf_fixture_type: &'a FixtureType,
        gdtf_dmx_mode: &'a DmxMode,

        fixtures: Vec<Fixture>,

        // Keeps track of how many siblings have been created at each depth of the geometry tree.
        // The top of the stack corresponds to the current parent whose children are being enumerated.
        sibling_count_stack: Vec<u32>,

        // Map a channel function (identified by geometry + indices + fixture id) to the
        // fixture id where it lives for quick lookup when resolving relations.
        channel_function_map: HashMap<ChannelFunctionId, FixtureId>,

        // Virtual channel functions are registered on the first pass, but their relations
        // depend on being able to find followers across the whole fixture set, so we store
        // them to resolve after the initial construction.
        unresolved_virtual_channels: Vec<(ChannelFunctionId, Attribute)>,

        defaults: HashSet<(Address, theymx::Value)>,
    }

    impl<'a> FixtureBuilder<'a> {
        pub fn new(
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
                channel_function_map: HashMap::new(),
                unresolved_virtual_channels: Vec::new(),
                defaults: HashSet::new(),
            }
        }

        pub(crate) fn build_fixture_tree(
            mut self,
        ) -> crate::Result<(Vec<Fixture>, HashSet<(Address, theymx::Value)>)> {
            // Find the root geometry for the chosen DMX mode and start the recursive building.
            let root_geometry = self.get_root_geometry()?.clone();
            let root_id = FixtureId::new(self.root_id);
            self.fixtures = self.fixtures_from_geometry(root_id, &root_geometry);

            // After building all fixtures and registering virtual channels, resolve their relations.
            self.resolve_virtual_channels();

            Ok((self.fixtures, self.defaults))
        }

        fn get_root_geometry(&self) -> crate::Result<&Geometry> {
            let Some(root_geometry) = self.gdtf_dmx_mode.geometry(self.gdtf_fixture_type) else {
                return Err(crate::Error::RootGeometryNotFound {
                    fixture_type_id: self.gdtf_fixture_type.fixture_type_id,
                    dmx_mode_name: self
                        .gdtf_dmx_mode
                        .name
                        .as_ref()
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "<no name>".to_string()),
                });
            };

            Ok(root_geometry)
        }

        fn fixtures_from_geometry(
            &mut self,
            sub_fixture_id: FixtureId,
            geometry: &Geometry,
        ) -> Vec<Fixture> {
            self.sibling_count_stack.push(0);

            let fixtures = match geometry {
                Geometry::Reference(reference) => {
                    self.fixture_from_reference_geometry(sub_fixture_id, reference)
                }
                geom => self.fixture_from_geometry(sub_fixture_id, geom),
            };

            self.sibling_count_stack.pop();

            fixtures
        }

        fn fixture_from_geometry(
            &mut self,
            sub_fixture_id: FixtureId,
            geometry: &Geometry,
        ) -> Vec<Fixture> {
            // Root fixture uses the provided fixture name, children use the geometry name.
            let name = if sub_fixture_id.len() == 1 {
                self.name.clone()
            } else {
                geometry.name().map(|n| n.to_string()).unwrap_or_else(|| "<no name>".to_string())
            };

            let geometry_name = match geometry.name() {
                Some(n) => n,
                None => return vec![], // Could also log or error, but for now skip
            };

            self.create_sub_fixture(sub_fixture_id, name, geometry_name, geometry_name, 0)
        }

        fn fixture_from_reference_geometry(
            &mut self,
            sub_fixture_id: FixtureId,
            reference_geometry: &ReferenceGeometry,
        ) -> Vec<Fixture> {
            // Reference geometries may introduce DMX address offsets via breaks.
            if reference_geometry.breaks.len() > 1 {
                log::warn!("multiple breaks not yet supported!");
            }

            let geometry_address_offset = match reference_geometry.breaks.first() {
                Some(b) => b.dmx_offset.absolute() as i32 - 1,
                None => 0,
            };

            let geometry_name = match reference_geometry.name() {
                Some(n) => n,
                None => return vec![], // Could also log or error, but for now skip
            };
            let referenced_geometry_name = match reference_geometry.geometry.as_ref() {
                Some(n) => n,
                None => return vec![], // Could also log or error, but for now skip
            };

            self.create_sub_fixture(
                sub_fixture_id,
                geometry_name.to_string(),
                geometry_name,
                referenced_geometry_name,
                geometry_address_offset,
            )
        }

        fn create_sub_fixture(
            &mut self,
            id: FixtureId,
            name: String,
            geometry: &Name,
            referenced_geometry: &Name,
            geometry_address_offset: i32,
        ) -> Vec<Fixture> {
            // Look up the nested geometry definition in the fixture type.
            let Some(referenced_geometry) =
                self.gdtf_fixture_type.nested_geometry(referenced_geometry)
            else {
                log::error!(
                    "Referenced geometry {:?} not found in fixture type {:?}",
                    referenced_geometry,
                    self.gdtf_fixture_type.fixture_type_id
                );
                return vec![];
            };

            // Build child fixtures first (they will push/pop their own sibling counters).
            let sub_fixtures = self.collect_child_fixtures(&id, referenced_geometry);
            // Collect only the immediate children ids for this fixture's metadata.
            let sub_ids = self.collect_direct_sub_ids(&id, &sub_fixtures);

            // Build channel functions for this referenced geometry (physical or virtual).
            let channel_functions = self.create_channel_functions(
                id,
                geometry,
                referenced_geometry.name().unwrap(),
                geometry_address_offset,
            );

            let gdtf_dmx_mode_name = self
                .gdtf_dmx_mode
                .name
                .as_ref()
                .map(|n| n.to_string())
                .unwrap_or_else(|| "<no mode name>".to_string());

            let mut fixtures = vec![Fixture {
                id,
                root_base_address: self.address,
                name,
                gdtf_fixture_type_id: self.gdtf_fixture_type.fixture_type_id,
                gdtf_dmx_mode: gdtf_dmx_mode_name,
                channel_functions,
                sub_ids,
            }];

            fixtures.extend(sub_fixtures);
            fixtures
        }

        fn collect_child_fixtures(&mut self, id: &FixtureId, geometry: &Geometry) -> Vec<Fixture> {
            let mut sub_fixtures = Vec::new();

            for child_geometry in geometry.children() {
                // Peek the current sibling count for this depth; it will be incremented only when
                // we actually add a fixture for this child.
                let sibling_count = {
                    let last = self.sibling_count_stack.last_mut().unwrap();
                    *last
                };

                let part = match FixtureIdPart::new(sibling_count + 1) {
                    Ok(part) => id.extended_with(part),
                    Err(err) => {
                        log::error!("invalid FixtureIdPart: {}", err);
                        continue; // Skip invalid id
                    }
                };
                let fixtures_for_child = self.fixtures_from_geometry(part, child_geometry);

                if fixtures_for_child.is_empty() {
                    continue;
                }

                // Only include this sub-fixture (and its descendants) if the top-level
                // fixture for this geometry has children or channel functions.
                let parent_fixture = &fixtures_for_child[0];
                if parent_fixture.channel_functions.is_empty() && parent_fixture.sub_ids.is_empty()
                {
                    continue;
                }

                // Only increment sibling count if we actually add a fixture
                let last = self.sibling_count_stack.last_mut().unwrap();
                *last += 1;

                sub_fixtures.extend(fixtures_for_child);
            }

            sub_fixtures
        }

        fn collect_direct_sub_ids(
            &self,
            id: &FixtureId,
            sub_fixtures: &[Fixture],
        ) -> Vec<FixtureId> {
            sub_fixtures
                .iter()
                .map(|f| f.id())
                .filter(|sub_id| sub_id.len() == id.len() + 1)
                .collect()
        }

        fn attribute_from_cf(&self, cf: &ChannelFunction) -> Option<Attribute> {
            cf.attribute(self.gdtf_fixture_type)
                .and_then(|attribute| attribute.name.as_ref())
                // Unwrapping here is safe, as from_str for Attribute cannot fail.
                .map(|attribute| Attribute::from_str(attribute).unwrap())
        }

        fn create_channel_functions(
            &mut self,
            id: FixtureId,
            geometry: &Name,
            referenced_geometry: &Name,
            geometry_address_offset: i32,
        ) -> HashMap<Attribute, FixtureChannelFunction> {
            // Find DMX channels that belong to the referenced geometry.
            let dmx_channels_with_geometry = self
                .gdtf_dmx_mode
                .dmx_channels
                .iter()
                .enumerate()
                .filter(|(_, dmx_channel)| dmx_channel.geometry == *referenced_geometry);

            let mut channel_functions = HashMap::new();

            for (c_ix, dmx_channel) in dmx_channels_with_geometry {
                for (lc_ix, logical_channel) in dmx_channel.logical_channels.iter().enumerate() {
                    // NOTE: filter out channel functions with a `NoFeature` attribute as they
                    //       interfere with computing DMX ranges.
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
                        // Compute the DMX range for this logical function: from current `dmx_from`
                        // up to (but not including) the next function's `dmx_from`, or to max.
                        let from = channel_function.dmx_from.into();
                        let to = filtered_channel_functions
                            .get(cf_ix + 1)
                            .map(|(_, cf)| ClampedValue::from(cf.dmx_from))
                            .unwrap_or_else(|| ClampedValue::new(ClampedValue::MAX));

                        let Some(attribute) = self.attribute_from_cf(channel_function) else {
                            // If we cannot parse an attribute, skip this channel function.
                            continue;
                        };

                        let cf_id = ChannelFunctionId {
                            fixture_id: id,
                            geometry: geometry.clone(),
                            channel_ix: c_ix,
                            logical_channel_ix: lc_ix,
                            channel_function_ix: *cf_ix,
                        };

                        // Determine whether this channel function is physical (has offsets) or virtual.
                        let kind = self.make_channel_function_kind(
                            dmx_channel,
                            &attribute,
                            cf_id.clone(),
                            geometry_address_offset,
                        );

                        let default = ClampedValue::from(channel_function.default);

                        // Collect the default values for the initial function.
                        if dmx_channel
                            .initial_function()
                            .is_some_and(|(_, cf)| cf == *channel_function)
                        {
                            match &kind {
                                FixtureChannelFunctionKind::Physical { addresses } => {
                                    let default_values = default.to_address_values(addresses);
                                    self.defaults.extend(default_values);
                                }
                                FixtureChannelFunctionKind::Virtual { .. } => {}
                            }
                        }

                        channel_functions.insert(
                            attribute,
                            FixtureChannelFunction { kind, min: from, max: to, default },
                        );

                        // Record where this channel function was created for relation lookup later.
                        self.channel_function_map.insert(cf_id, id);
                    }
                }
            }

            channel_functions
        }

        fn make_channel_function_kind(
            &mut self,
            dmx_channel: &DmxChannel,
            attribute: &Attribute,
            cf_id: ChannelFunctionId,
            geometry_address_offset: i32,
        ) -> FixtureChannelFunctionKind {
            match &dmx_channel.offset {
                Some(offsets) => {
                    // Physical channel: map each offset to an absolute DMX address.
                    let addresses = offsets
                        .iter()
                        .filter_map(|o| {
                            match self.address.with_channel_offset(geometry_address_offset + o - 1)
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
                    // Virtual channel: register for resolution later and return an empty relation set.
                    self.register_virtual_channel(*attribute, cf_id);
                    FixtureChannelFunctionKind::Virtual { relations: vec![] }
                }
            }
        }

        fn register_virtual_channel(&mut self, attribute: Attribute, cf_id: ChannelFunctionId) {
            self.unresolved_virtual_channels.push((cf_id, attribute));
        }

        fn resolve_virtual_channels(&mut self) {
            // Iterate over virtual channels we registered during the first pass and populate
            // their relation lists by inspecting the DMX mode relations and mapping them to
            // fixtures in our constructed tree.
            for (cf_id, virtual_attribute) in &self.unresolved_virtual_channels {
                let Some(dmx_channel) = self.gdtf_dmx_mode.dmx_channels.get(cf_id.channel_ix)
                else {
                    continue;
                };

                let relations = self.get_relations_for_dmx_channel(&cf_id.geometry, dmx_channel);

                let Some(fixture) = self.fixtures.iter_mut().find(|f| f.id() == cf_id.fixture_id)
                else {
                    continue;
                };

                let Some(virtual_channel_function) =
                    fixture.channel_functions.get_mut(virtual_attribute)
                else {
                    continue;
                };

                // Replace the empty relation vector with the resolved relations.
                virtual_channel_function.kind = FixtureChannelFunctionKind::Virtual { relations };
            }
        }

        /// Build relation structures for the provided DMX channel by inspecting DMX mode relations.
        fn get_relations_for_dmx_channel(
            &self,
            geometry: &Name,
            dmx_channel: &DmxChannel,
        ) -> Vec<Relation> {
            let mut channel_relations = Vec::new();

            let relations = self.gdtf_dmx_mode.relations.iter().filter(|relation| {
                relation
                    .master(self.gdtf_dmx_mode)
                    .is_some_and(|master| master.name() == dmx_channel.name())
            });

            for relation in relations {
                let Some((_, _, follower_channel_function)) = relation.follower(self.gdtf_dmx_mode)
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

                let Some(fixture_id) =
                    self.fixture_id_for_channel_function(geometry, follower_channel_function)
                else {
                    log::warn!(
                        "could not find fixture id for follower channel function {}",
                        follower_channel_function.name.as_deref().unwrap_or("<no name>")
                    );
                    continue;
                };

                let Some(attribute) = self.attribute_from_cf(follower_channel_function) else {
                    continue;
                };

                channel_relations.push(Relation::new(kind, fixture_id, attribute));
            }

            channel_relations
        }

        /// Find the `FixtureId` that corresponds to the provided channel function pointer
        /// and geometry. We need pointer equality because the same `ChannelFunction` instances
        /// (from the DMX mode) are referenced in relations.
        fn fixture_id_for_channel_function(
            &self,
            geometry: &Name,
            target_channel_function: &ChannelFunction,
        ) -> Option<FixtureId> {
            for (c_ix, dmx_channel) in self.gdtf_dmx_mode.dmx_channels.iter().enumerate() {
                for (lc_ix, logical_channel) in dmx_channel.logical_channels.iter().enumerate() {
                    for (cf_ix, channel_function) in
                        logical_channel.channel_functions.iter().enumerate()
                    {
                        if !std::ptr::eq(target_channel_function, channel_function) {
                            continue;
                        }

                        // Look up the recorded fixture id for the matching channel function id.
                        if let Some((_, fixture_id)) =
                            self.channel_function_map.iter().find(|(id, _)| {
                                &id.geometry == geometry
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

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct ChannelFunctionId {
        fixture_id: FixtureId,
        geometry: Name,
        channel_ix: usize,
        logical_channel_ix: usize,
        channel_function_ix: usize,
    }

    impl From<gdtf::values::DmxValue> for ClampedValue {
        fn from(value: gdtf::values::DmxValue) -> Self {
            let len: u8 = value.bytes().into();
            let raw = value.to(len);
            let max_value = 2_u64.saturating_pow(len as u32 * 8) - 1;
            let floating_value = raw as f32 / max_value as f32;
            ClampedValue::new(floating_value)
        }
    }
}
