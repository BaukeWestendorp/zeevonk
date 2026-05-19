mod fixture_builder;

use std::collections::{BTreeMap, BTreeSet};

use rigger::gdtf::dmx::DmxMode;
use rigger::gdtf::{FixtureTypeId, Gdtf, Name};

use crate::project::builder::stage::fixture_builder::FixtureBuilder;
use crate::project::file::ProjectFile;
use crate::project::stage::{Fixture, FixtureChannelFunctionKind, FixtureId, FixtureIdPart, Stage};
use crate::value::AttributeValues;

pub fn from_file(file: &ProjectFile) -> crate::Result<Stage> {
    let mut stage =
        Stage { fixtures: BTreeMap::new(), default_attribute_values: AttributeValues::new() };

    let gdtfs = load_gdtfs(file)?;

    for fixture_def in &file.patch.fixtures {
        build_patch_fixture(&mut stage, &gdtfs, fixture_def)?;
    }

    normalize(&mut stage);

    Ok(stage)
}

fn build_patch_fixture(
    stage: &mut Stage,
    gdtfs: &BTreeMap<FixtureTypeId, Gdtf>,
    fixture_def: &crate::project::FixtureDefinition,
) -> crate::Result<()> {
    let (fixture_type, dmx_mode) = fixture_type_and_mode(gdtfs, fixture_def)?;

    let builder = FixtureBuilder::new(
        fixture_def.root_id,
        fixture_def.name.to_owned(),
        fixture_def.address,
        fixture_type,
        dmx_mode,
    );

    let (built_fixtures, defaults) = builder.build_fixture_tree()?;
    for built_fixture in built_fixtures {
        stage.fixtures.insert(built_fixture.id(), built_fixture);
    }

    stage.default_attribute_values.extend(defaults);

    Ok(())
}

fn fixture_type_and_mode<'a>(
    gdtfs: &'a BTreeMap<FixtureTypeId, Gdtf>,
    fixture_def: &crate::project::FixtureDefinition,
) -> crate::Result<(&'a Gdtf, &'a DmxMode)> {
    let fixture_type =
        gdtfs.get(&FixtureTypeId::new(fixture_def.kind.gdtf_fixture_type_id)).ok_or_else(|| {
            crate::Error::FixtureTypeNotFound { id: fixture_def.kind.gdtf_fixture_type_id }
        })?;

    let dmx_mode = fixture_type
        .dmx_mode(&Name::new(&fixture_def.kind.gdtf_dmx_mode))
        .ok_or(crate::Error::DmxModeNotFound)?;

    Ok((fixture_type, dmx_mode))
}

fn load_gdtfs(file: &ProjectFile) -> crate::Result<BTreeMap<FixtureTypeId, Gdtf>> {
    let mut gdtfs = BTreeMap::new();

    for gdtf_file_path in &file.patch.gdtf_file_paths {
        let gdtf = Gdtf::from_archive(gdtf_file_path);
        gdtfs.insert(gdtf.fixture_type_id(), gdtf);
    }

    Ok(gdtfs)
}

fn normalize(stage: &mut Stage) {
    prune_empty(stage);
    collapse(stage);
    renumber(stage);
}

fn prune_empty(stage: &mut Stage) {
    let fixture_ids: Vec<FixtureId> = stage.fixtures.keys().cloned().collect();
    let mut keep: BTreeSet<FixtureId> = BTreeSet::new();

    for id in &fixture_ids {
        if id.is_root() {
            continue;
        };

        let has_channel_functions =
            stage.fixtures.get(id).is_some_and(|f| !f.channel_functions.is_empty());

        if !has_channel_functions {
            continue;
        }

        let mut current = Some(id.clone());
        while let Some(cur) = current {
            if !keep.insert(cur.clone()) {
                break;
            }
            current = stage.parent_id(&cur);
        }
    }

    stage.fixtures.retain(|id, _| keep.contains(id));
    stage.default_attribute_values.retain_fixtures(&keep);

    let existing: BTreeSet<FixtureId> = stage.fixtures.keys().cloned().collect();
    for fixture in stage.fixtures.values_mut() {
        fixture.child_ids.retain(|cid| existing.contains(cid));
    }
}

fn collapse(stage: &mut Stage) {
    loop {
        let fixture_ids: Vec<FixtureId> = stage.fixtures.keys().cloned().collect();

        let mut changed = false;

        for parent_id in fixture_ids {
            let Some(parent) = stage.fixtures.get(&parent_id) else {
                continue;
            };

            if parent.child_ids.is_empty() {
                continue;
            }

            let child_ids_with_channel_functions: Vec<FixtureId> = parent
                .child_ids
                .iter()
                .filter_map(|cid| {
                    stage
                        .fixtures
                        .get(cid)
                        .is_some_and(|c| !c.channel_functions.is_empty())
                        .then(|| cid.clone())
                })
                .collect();

            if child_ids_with_channel_functions.len() != 1 {
                continue;
            }

            let child_id = child_ids_with_channel_functions[0].clone();

            let Some(child) = stage.fixtures.get(&child_id) else {
                continue;
            };

            let child_direct_children = child.child_ids.clone();

            let child_channel_functions = stage
                .fixtures
                .get_mut(&child_id)
                .map(|c| std::mem::take(&mut c.channel_functions))
                .unwrap_or_default();

            for attribute in child_channel_functions.keys() {
                stage
                    .default_attribute_values
                    .move_attribute_value(&child_id, parent_id, attribute);
            }

            let child_highlight_values = stage
                .fixtures
                .get_mut(&child_id)
                .map(|c| std::mem::take(&mut c.highlight_values))
                .unwrap_or_default();

            if child_channel_functions.is_empty() && child_highlight_values.is_empty() {
                continue;
            }

            if let Some(parent) = stage.fixtures.get_mut(&parent_id) {
                parent.channel_functions.extend(child_channel_functions);
                parent.highlight_values.extend(child_highlight_values);

                parent.child_ids.retain(|cid| cid != &child_id);

                for grandchild_id in child_direct_children {
                    if !parent.child_ids.contains(&grandchild_id) {
                        parent.child_ids.push(grandchild_id);
                    }
                }
            }

            stage.fixtures.remove(&child_id);

            for fixture in stage.fixtures.values_mut() {
                for cf in fixture.channel_functions.values_mut() {
                    if let FixtureChannelFunctionKind::Virtual { relations } = &mut cf.kind {
                        for rel in relations.iter_mut() {
                            if rel.fixture_id == child_id {
                                rel.fixture_id = parent_id;
                            }
                        }
                    }
                }
            }

            changed = true;
        }

        if !changed {
            break;
        }
    }
}

fn renumber(stage: &mut Stage) {
    loop {
        let root_ids: Vec<FixtureId> =
            stage.fixtures.keys().filter(|id| id.len() == 1).cloned().collect();

        let mut id_map: BTreeMap<FixtureId, FixtureId> = BTreeMap::new();
        let mut stack: Vec<FixtureId> = Vec::new();

        for root_id in root_ids {
            id_map.insert(root_id, root_id);
            stack.push(root_id);
        }

        while let Some(old_parent_id) = stack.pop() {
            let Some(parent) = stage.fixtures.get(&old_parent_id) else {
                continue;
            };

            let mut children: Vec<FixtureId> = parent
                .child_ids
                .iter()
                .filter(|cid| stage.fixtures.contains_key(*cid))
                .cloned()
                .collect();

            children.sort();

            for (ix, old_child_id) in children.into_iter().enumerate() {
                let Ok(new_part) = FixtureIdPart::new((ix as u32) + 1) else {
                    continue;
                };

                let old_mapped_parent =
                    id_map.get(&old_parent_id).cloned().unwrap_or(old_parent_id.clone());
                let new_child_id = old_mapped_parent.extended_with(new_part);

                if new_child_id != old_child_id {
                    id_map.insert(old_child_id.clone(), new_child_id.clone());
                } else {
                    id_map.insert(old_child_id.clone(), old_child_id.clone());
                }

                stack.push(old_child_id);
            }
        }

        let changed = id_map.iter().any(|(old, new)| old != new);
        if !changed {
            break;
        }

        let old_fixtures = std::mem::take(&mut stage.fixtures);
        let mut new_fixtures: BTreeMap<FixtureId, Fixture> = BTreeMap::new();

        for (old_id, mut fixture) in old_fixtures {
            let new_id = id_map.get(&old_id).cloned().unwrap_or(old_id);

            fixture.id = new_id;

            for cid in fixture.child_ids.iter_mut() {
                if let Some(mapped) = id_map.get(cid) {
                    *cid = mapped.clone();
                }
            }

            for cf in fixture.channel_functions.values_mut() {
                if let FixtureChannelFunctionKind::Virtual { relations } = &mut cf.kind {
                    for rel in relations.iter_mut() {
                        if let Some(mapped) = id_map.get(&rel.fixture_id) {
                            rel.fixture_id = *mapped;
                        }
                    }
                }
            }

            new_fixtures.insert(fixture.id(), fixture);
        }

        stage.fixtures = new_fixtures;
        stage.default_attribute_values.remap_fixture_ids(&id_map);
    }
}
