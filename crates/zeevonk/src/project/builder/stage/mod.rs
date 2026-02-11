mod fixture_builder;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use gdtf::dmx_mode::DmxMode;
use gdtf::fixture_type::FixtureType;
use uuid::Uuid;

use crate::project::builder::stage::fixture_builder::FixtureBuilder;
use crate::project::file::ProjectFile;
use crate::project::stage::{Fixture, FixtureChannelFunctionKind, FixtureId, FixtureIdPart, Stage};
use crate::theymx::Multiverse;

pub fn from_file(file: &ProjectFile) -> crate::Result<Stage> {
    let mut stage = Stage { fixtures: BTreeMap::new(), default_multiverse: Multiverse::new() };

    let fixture_types = load_fixture_types(file)?;

    for fixture_def in &file.patch.fixtures {
        build_patch_fixture(&mut stage, &fixture_types, fixture_def)?;
    }

    normalize(&mut stage);

    Ok(stage)
}

fn build_patch_fixture(
    stage: &mut Stage,
    fixture_types: &BTreeMap<Uuid, FixtureType>,
    fixture_def: &crate::project::file::patch::FixtureDefinition,
) -> crate::Result<()> {
    let (fixture_type, dmx_mode) = fixture_type_and_mode(fixture_types, fixture_def)?;

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
    for (address, value) in defaults {
        stage.default_multiverse.set_value(&address, value);
    }

    Ok(())
}

fn fixture_type_and_mode<'a>(
    fixture_types: &'a BTreeMap<Uuid, FixtureType>,
    fixture_def: &crate::project::file::patch::FixtureDefinition,
) -> crate::Result<(&'a FixtureType, &'a DmxMode)> {
    let fixture_type =
        fixture_types.get(&fixture_def.kind.gdtf_fixture_type_id).ok_or_else(|| {
            crate::Error::FixtureTypeNotFound { id: fixture_def.kind.gdtf_fixture_type_id }
        })?;

    let dmx_mode = fixture_type
        .dmx_mode(&fixture_def.kind.gdtf_dmx_mode)
        .ok_or(crate::Error::DmxModeNotFound)?;

    Ok((fixture_type, dmx_mode))
}

fn load_fixture_types(file: &ProjectFile) -> crate::Result<BTreeMap<Uuid, FixtureType>> {
    let mut fixture_types = BTreeMap::new();

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
            fixture_types.insert(fixture_type.fixture_type_id, fixture_type);
        }
    }

    Ok(fixture_types)
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

            if child_channel_functions.is_empty() {
                continue;
            }

            if let Some(parent_mut) = stage.fixtures.get_mut(&parent_id) {
                parent_mut.channel_functions.extend(child_channel_functions);

                parent_mut.child_ids.retain(|cid| cid != &child_id);

                for grandchild_id in child_direct_children {
                    if !parent_mut.child_ids.contains(&grandchild_id) {
                        parent_mut.child_ids.push(grandchild_id);
                    }
                }
            }

            stage.fixtures.remove(&child_id);

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
    }
}
