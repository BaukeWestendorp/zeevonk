// FIXME: For some reason the use of these functions in the benchmarks do not
// mark them as used here.
#![allow(dead_code)]

use std::path::Path;

use zeevonk::attr::Attribute;
use zeevonk::project::Project;
use zeevonk::project::file::ProjectFile;
use zeevonk::project::file::patch::{FixtureDefinition, FixtureKindDefinition, Patch};
use zeevonk::project::stage::{FixtureIdPart, Stage};
use zeevonk::theymx::Address;
use zeevonk::value::{AttributeValues, ClampedValue};

pub fn generate_project(
    n_fixtures: u32,
    path: &str,
    gdtf_fixture_type_id: &str,
    gdtf_dmx_mode: &str,
    name: &str,
) -> zeevonk::Result<Project> {
    let fixtures = (0..n_fixtures)
        .map(|n| FixtureDefinition {
            name: (*name).to_string(),
            root_id: FixtureIdPart::new(n + 1).unwrap(),
            address: Address::from_absolute(n + 1).unwrap(),
            kind: FixtureKindDefinition {
                gdtf_fixture_type_id: gdtf_fixture_type_id.parse().unwrap(),
                gdtf_dmx_mode: gdtf_dmx_mode.to_string(),
            },
        })
        .collect();

    let gdtf_file_paths = vec![Path::new(path).into()];

    let project_file =
        ProjectFile { patch: Patch { gdtf_file_paths, fixtures }, ..Default::default() };

    zeevonk::project::builder::from_file(project_file)
}

pub fn build_values(stage: &Stage, attrs: &[(Attribute, ClampedValue)]) -> AttributeValues {
    let mut values = AttributeValues::new();
    for (fid, _) in stage.fixtures() {
        for (child_fid, _) in stage.children(fid) {
            for &(attr, v) in attrs {
                values.set(*child_fid, attr, v);
            }
        }
    }
    values
}
