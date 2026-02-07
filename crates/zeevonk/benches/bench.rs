use std::path::Path;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use theymx::Address;
use zeevonk::attr::Attribute;
use zeevonk::project::Project;
use zeevonk::project::file::ProjectFile;
use zeevonk::project::file::patch::{FixtureDefinition, FixtureKindDefinition, Patch};
use zeevonk::project::stage::{FixtureIdPart, Stage};
use zeevonk::theymx::Multiverse;
use zeevonk::value::{AttributeValues, ClampedValue};

criterion_main!(resolving);
criterion_group!(resolving, bench_resolver);

fn bench_resolver(c: &mut Criterion) {
    run_scenarios(
        c,
        "resolve_dimmers",
        &[
            50, 100, 150, 200, 250, 300, 350, 400, 450, 500, 550, 600, 650, 700, 750, 800, 850,
            900, 950, 1000,
        ],
        &[
            Scenario {
                name: "8bit",
                fixture: FixtureConfig::GenericDimmer {
                    gdtf_file_path: "benches/gdtf/Generic@Dimmer@8_and_16bit_Generic_Dimmer.gdtf",
                    gdtf_fixture_type_id: "B4DAFF6B-3E52-451B-AFDB-E6C94C64F85D",
                    gdtf_dmx_mode: "Default",
                    name: "Dimmer",
                },
                attributes: vec![(Attribute::Dimmer, ClampedValue::new(0.5))],
            },
            Scenario {
                name: "16bit",
                fixture: FixtureConfig::GenericDimmer {
                    gdtf_file_path: "benches/gdtf/Generic@Dimmer@8_and_16bit_Generic_Dimmer.gdtf",
                    gdtf_fixture_type_id: "B4DAFF6B-3E52-451B-AFDB-E6C94C64F85D",
                    gdtf_dmx_mode: "16 Bit",
                    name: "Dimmer",
                },
                attributes: vec![(Attribute::Dimmer, ClampedValue::new(0.5))],
            },
        ],
    );

    run_scenarios(
        c,
        "resolve_rgbw",
        &[
            50, 100, 150, 200, 250, 300, 350, 400, 450, 500, 550, 600, 650, 700, 750, 800, 850,
            900, 950, 1000,
        ],
        &[Scenario {
            name: "8bit",
            fixture: FixtureConfig::GenericDimmer {
                gdtf_file_path: "benches/gdtf/Generic@RGBW8@added_white_channel.gdtf",
                gdtf_fixture_type_id: "E6E44F65-4F1D-4B62-B614-9AB1F6C0C2D1",
                gdtf_dmx_mode: "Default",
                name: "Dimmer",
            },
            attributes: vec![
                (Attribute::ColorAddR, ClampedValue::new(0.5)),
                (Attribute::ColorAddG, ClampedValue::new(0.5)),
                (Attribute::ColorAddB, ClampedValue::new(0.5)),
                (Attribute::ColorAddW, ClampedValue::new(0.5)),
            ],
        }],
    );
}

fn run_scenarios(
    c: &mut Criterion,
    group_name: &str,
    fixture_counts: &[u32],
    scenarios: &[Scenario],
) {
    let mut group = c.benchmark_group(group_name);

    for scenario in scenarios {
        for &n_fixtures in fixture_counts {
            let project = generate_project(n_fixtures, &scenario.fixture).unwrap();
            let values = build_values(project.stage(), &scenario.attributes);

            group.bench_with_input(
                BenchmarkId::new(scenario.name, n_fixtures),
                &n_fixtures,
                |b, &_n| {
                    let mut multiverse = Multiverse::new();
                    b.iter(|| {
                        zeevonk::resolver::resolve(&values, project.stage(), &mut multiverse);
                        std::hint::black_box(&mut multiverse);
                    });
                },
            );
        }
    }

    group.finish();
}

fn build_values(stage: &Stage, attrs: &[(Attribute, ClampedValue)]) -> AttributeValues {
    let mut values = AttributeValues::new();
    for (fid, _) in stage.fixtures() {
        for (sub_fid, _) in stage.sub_fixtures(fid) {
            for &(attr, v) in attrs {
                values.set(*sub_fid, attr, v);
            }
        }
    }
    values
}

fn generate_project(n_fixtures: u32, config: &FixtureConfig) -> zeevonk::Result<Project> {
    let fixtures = (0..n_fixtures)
        .map(|n| match config {
            FixtureConfig::GenericDimmer { gdtf_fixture_type_id, gdtf_dmx_mode, name, .. } => {
                FixtureDefinition {
                    name: (*name).to_string(),
                    root_id: FixtureIdPart::new(n + 1).unwrap(),
                    address: Address::from_absolute(n + 1).unwrap(),
                    kind: FixtureKindDefinition {
                        gdtf_fixture_type_id: (*gdtf_fixture_type_id).parse().unwrap(),
                        gdtf_dmx_mode: (*gdtf_dmx_mode).to_string(),
                    },
                }
            }
        })
        .collect();

    let gdtf_file_paths = match config {
        FixtureConfig::GenericDimmer { gdtf_file_path, .. } => {
            vec![Path::new(gdtf_file_path).into()]
        }
    };

    let project_file =
        ProjectFile { patch: Patch { gdtf_file_paths, fixtures }, ..Default::default() };

    zeevonk::project_builder::from_file(project_file)
}

struct Scenario {
    name: &'static str,
    fixture: FixtureConfig,
    attributes: Vec<(Attribute, ClampedValue)>,
}

enum FixtureConfig {
    GenericDimmer {
        gdtf_file_path: &'static str,
        gdtf_fixture_type_id: &'static str,
        gdtf_dmx_mode: &'static str,
        name: &'static str,
    },
}
