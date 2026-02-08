use std::path::Path;

use criterion::BatchSize;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use theymx::Address;
use theymx::Multiverse;
use zeevonk::attr::Attribute;
use zeevonk::project::Project;
use zeevonk::project::file::ProjectFile;
use zeevonk::project::file::patch::FixtureDefinition;
use zeevonk::project::file::patch::FixtureKindDefinition;
use zeevonk::project::file::patch::Patch;
use zeevonk::project::stage::FixtureIdPart;
use zeevonk::project::stage::Stage;
use zeevonk::value::AttributeValues;
use zeevonk::value::ClampedValue;

criterion_group!(
    benches,
    bench_fixture_count_scaling,
    bench_attribute_complexity,
    bench_realistic_single_universe
);
criterion_main!(benches);

fn bench_fixture_count_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("Scaling Fixture Count");

    group.sample_size(100);

    let counts = [50, 500, 1000];

    for count in counts {
        let path = "benches/gdtf/Generic@Dimmer@8_and_16bit_Generic_Dimmer.gdtf";
        let project = generate_project(
            count,
            path,
            "B4DAFF6B-3E52-451B-AFDB-E6C94C64F85D",
            "Default",
            "Dimmer",
        )
        .unwrap();
        let values = build_values(project.stage(), &[(Attribute::Dimmer, ClampedValue::new(0.5))]);

        group.bench_with_input(BenchmarkId::new("Dimmer 8 Bit", count), &count, |b, &_| {
            b.iter_with_setup(
                || Multiverse::new(),
                |mut multiverse| {
                    zeevonk::resolver::resolve(&values, project.stage(), &mut multiverse);
                },
            )
        });
    }

    for count in counts {
        let path = "benches/gdtf/Generic@Dimmer@8_and_16bit_Generic_Dimmer.gdtf";
        let project = generate_project(
            count,
            path,
            "B4DAFF6B-3E52-451B-AFDB-E6C94C64F85D",
            "Default",
            "16 Bit",
        )
        .unwrap();
        let values = build_values(project.stage(), &[(Attribute::Dimmer, ClampedValue::new(0.5))]);

        group.bench_with_input(BenchmarkId::new("Dimmer 16 Bit", count), &count, |b, &_| {
            b.iter_batched(
                || Multiverse::new(),
                |mut multiverse| {
                    zeevonk::resolver::resolve(&values, project.stage(), &mut multiverse);
                },
                BatchSize::PerIteration,
            );
        });
    }

    for count in counts {
        let path = "benches/gdtf/Generic@RGBW8@added_white_channel.gdtf";
        let project = generate_project(
            count,
            path,
            "E6E44F65-4F1D-4B62-B614-9AB1F6C0C2D1",
            "Default",
            "RGBW",
        )
        .unwrap();
        let values = build_values(
            project.stage(),
            &[
                (Attribute::ColorAddR, ClampedValue::new(0.5)),
                (Attribute::ColorAddG, ClampedValue::new(0.5)),
                (Attribute::ColorAddB, ClampedValue::new(0.5)),
                (Attribute::ColorAddW, ClampedValue::new(0.5)),
            ],
        );

        group.bench_with_input(BenchmarkId::new("RGBW 8 Bit", count), &count, |b, &_| {
            b.iter_batched(
                || Multiverse::new(),
                |mut multiverse| {
                    zeevonk::resolver::resolve(&values, project.stage(), &mut multiverse);
                },
                BatchSize::PerIteration,
            );
        });
    }

    group.finish();
}

fn bench_attribute_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("Scaling Complexity");

    let fixture_count = 100;

    #[rustfmt::skip]
    let complexities = {
        use zeevonk::attr::Attribute as A;
        [
            ("5 channels",  vec![A::Dimmer, A::Pan, A::Tilt, A::PanRotate, A::TiltRotate]),
            ("10 channels", vec![A::Dimmer, A::Pan, A::Tilt, A::PanRotate, A::TiltRotate, A::PositionEffect, A::PositionEffectRate, A::PositionEffectFade, A::XyzX, A::XyzY,]),
            ("15 channels", vec![A::Dimmer, A::Pan, A::Tilt, A::PanRotate, A::TiltRotate, A::PositionEffect, A::PositionEffectRate, A::PositionEffectFade, A::XyzX, A::XyzY, A::XyzZ, A::RotX, A::RotY, A::RotZ, A::ScaleX]),
            ("20 channels", vec![A::Dimmer, A::Pan, A::Tilt, A::PanRotate, A::TiltRotate, A::PositionEffect, A::PositionEffectRate, A::PositionEffectFade, A::XyzX, A::XyzY, A::XyzZ, A::RotX, A::RotY, A::RotZ, A::ScaleX, A::ScaleY, A::ScaleZ, A::ScaleXYZ, A::PlayMode, A::PlayBegin]),
            ("25 channels", vec![A::Dimmer, A::Pan, A::Tilt, A::PanRotate, A::TiltRotate, A::PositionEffect, A::PositionEffectRate, A::PositionEffectFade, A::XyzX, A::XyzY, A::XyzZ, A::RotX, A::RotY, A::RotZ, A::ScaleX, A::ScaleY, A::ScaleZ, A::ScaleXYZ, A::PlayMode, A::PlayBegin, A::PlayEnd, A::PlaySpeed, A::ColorAddR, A::ColorAddG, A::ColorAddB]),
            ("30 channels", vec![A::Dimmer, A::Pan, A::Tilt, A::PanRotate, A::TiltRotate, A::PositionEffect, A::PositionEffectRate, A::PositionEffectFade, A::XyzX, A::XyzY, A::XyzZ, A::RotX, A::RotY, A::RotZ, A::ScaleX, A::ScaleY, A::ScaleZ, A::ScaleXYZ, A::PlayMode, A::PlayBegin, A::PlayEnd, A::PlaySpeed, A::ColorAddR, A::ColorAddG, A::ColorAddB, A::ColorAddRY, A::ColorAddW, A::ColorAddWW, A::ColorAddCW, A::ColorWheelReset])
        ]
    };

    for (name, attrs) in complexities {
        let path = "benches/gdtf/Zeevonk@Benchmark@Added_30ch_Mode.gdtf";
        let project = generate_project(
            fixture_count,
            path,
            "9125433D-A327-434E-95C5-D116FCBFB7D9",
            "30ch",
            "Bench",
        )
        .unwrap();

        let attr_values: Vec<(Attribute, ClampedValue)> =
            attrs.iter().map(|&a| (a, ClampedValue::new(0.5))).collect();

        let values = build_values(project.stage(), &attr_values);

        group.bench_function(BenchmarkId::new("Attributes", name), |b| {
            b.iter_batched(
                || Multiverse::new(),
                |mut multiverse| {
                    zeevonk::resolver::resolve(&values, project.stage(), &mut multiverse);
                },
                BatchSize::PerIteration,
            );
        });
    }

    group.finish();
}

fn bench_realistic_single_universe(c: &mut Criterion) {
    let project = zeevonk::project_builder::from_file(
        ProjectFile::load_from_folder(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("benches/projects/realistic_single_universe"),
        )
        .unwrap(),
    )
    .unwrap();

    let mut values = AttributeValues::new();

    let mut set_all = |addr: String, attr: Attribute| {
        values.set(addr.parse().unwrap(), attr, ClampedValue::new(0.5));
    };

    for n in 101..=104 {
        set_all(format!("{n}.1"), Attribute::Pan);
        set_all(format!("{n}.1.1"), Attribute::Color(1));
        set_all(format!("{n}.1.1"), Attribute::Shutter(1));
        set_all(format!("{n}.1.1"), Attribute::Dimmer);
        set_all(format!("{n}.1.1"), Attribute::Gobo(1));
        set_all(format!("{n}.1.1"), Attribute::Prism(1));
        set_all(format!("{n}.1.1"), Attribute::PrismPos(1));
        set_all(format!("{n}.1.1"), Attribute::EffectsPos(1));
        set_all(format!("{n}.1.1"), Attribute::Frost(1));
        set_all(format!("{n}.1.1"), Attribute::Focus(1));
        set_all(format!("{n}.1.1"), Attribute::Tilt);
        set_all(format!("{n}.1.1"), Attribute::PositionMSpeed);
        set_all(format!("{n}.1.1"), Attribute::FixtureGlobalReset);
        set_all(format!("{n}.1.1"), Attribute::LampControl);
    }

    for n in 201..=208 {
        set_all(format!("{n}"), Attribute::PositionMSpeed);
        set_all(format!("{n}"), Attribute::Control(1));
        set_all(format!("{n}.1"), Attribute::Pan);
        set_all(format!("{n}.1.1"), Attribute::Tilt);
        set_all(format!("{n}.1.1"), Attribute::ColorAddR);
        set_all(format!("{n}.1.1"), Attribute::ColorAddG);
        set_all(format!("{n}.1.1"), Attribute::ColorAddB);
        set_all(format!("{n}.1.1"), Attribute::ColorAddW);
        set_all(format!("{n}.1.1"), Attribute::Cto);
        set_all(format!("{n}.1.1"), Attribute::Color(1));
        set_all(format!("{n}.1.1"), Attribute::Zoom);
        set_all(format!("{n}.1.1"), Attribute::Shutter(1));
        set_all(format!("{n}.1.1"), Attribute::Dimmer);
    }

    for n in 301..=312 {
        set_all(format!("{n}.1"), Attribute::Pan);
        set_all(format!("{n}.1"), Attribute::PanRotate);
        set_all(format!("{n}.1.1"), Attribute::Tilt);
        set_all(format!("{n}.1.1"), Attribute::TiltRotate);
        set_all(format!("{n}.1.1"), Attribute::ColorAddR);
        set_all(format!("{n}.1.1"), Attribute::ColorAddG);
        set_all(format!("{n}.1.1"), Attribute::ColorAddB);
        set_all(format!("{n}.1.1"), Attribute::Gobo(1));
        set_all(format!("{n}.1.1"), Attribute::Prism(1));
        set_all(format!("{n}.1.1"), Attribute::PrismPos(1));
        set_all(format!("{n}.1.1"), Attribute::Prism(2));
        set_all(format!("{n}.1.1"), Attribute::PrismPos(2));
        set_all(format!("{n}.1.1"), Attribute::Shutter(1));
        set_all(format!("{n}.1.1"), Attribute::Dimmer);
        set_all(format!("{n}.1.1"), Attribute::Focus(1));
        set_all(format!("{n}.1.1"), Attribute::Frost(1));
        set_all(format!("{n}.1.1"), Attribute::DimmerMode);
        set_all(format!("{n}.1.1"), Attribute::PositionMSpeed);
        set_all(format!("{n}.1.1"), Attribute::Function);
    }

    for n in 401..=404 {
        set_all(format!("{n}.1"), Attribute::Pan);
        set_all(format!("{n}.1"), Attribute::PanRotate);
        set_all(format!("{n}.1.1"), Attribute::Tilt);
        set_all(format!("{n}.1.1"), Attribute::TiltRotate);
        set_all(format!("{n}.1.1"), Attribute::ColorAddR);
        set_all(format!("{n}.1.1"), Attribute::ColorAddG);
        set_all(format!("{n}.1.1"), Attribute::ColorAddB);
        set_all(format!("{n}.1.1"), Attribute::Gobo(1));
        set_all(format!("{n}.1.1"), Attribute::Prism(1));
        set_all(format!("{n}.1.1"), Attribute::PrismPos(1));
        set_all(format!("{n}.1.1"), Attribute::Prism(2));
        set_all(format!("{n}.1.1"), Attribute::PrismPos(2));
        set_all(format!("{n}.1.1"), Attribute::Shutter(1));
        set_all(format!("{n}.1.1"), Attribute::Dimmer);
        set_all(format!("{n}.1.1"), Attribute::Focus(1));
        set_all(format!("{n}.1.1"), Attribute::Frost(1));
        set_all(format!("{n}.1.1"), Attribute::DimmerMode);
        set_all(format!("{n}.1.1"), Attribute::PositionMSpeed);
        set_all(format!("{n}.1.1"), Attribute::Function);
    }

    for n in 501..=505 {
        set_all(format!("{n}.1"), Attribute::Tilt);
        set_all(format!("{n}.1"), Attribute::Control(1));
        set_all(format!("{n}.1.1"), Attribute::Dimmer);
        set_all(format!("{n}.1.1"), Attribute::StrobeDuration);
        set_all(format!("{n}.1.1"), Attribute::StrobeRate);
        set_all(format!("{n}.1.1"), Attribute::StrobeModeStrobe);
        set_all(format!("{n}.1.2"), Attribute::Dimmer);
        set_all(format!("{n}.1.2"), Attribute::StrobeDuration);
        set_all(format!("{n}.1.2"), Attribute::StrobeRate);
        set_all(format!("{n}.1.2"), Attribute::StrobeModeStrobe);
        set_all(format!("{n}.1.2"), Attribute::ColorAddR);
        set_all(format!("{n}.1.2"), Attribute::ColorAddG);
        set_all(format!("{n}.1.2"), Attribute::ColorAddB);
    }

    set_all("601.1".to_string(), Attribute::Dimmer);

    c.bench_function("Realistic Single Universe", |b| {
        b.iter_batched(
            || Multiverse::new(),
            |mut multiverse| {
                zeevonk::resolver::resolve(&values, project.stage(), &mut multiverse);
            },
            BatchSize::PerIteration,
        );
    });
}

fn generate_project(
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

    zeevonk::project_builder::from_file(project_file)
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
