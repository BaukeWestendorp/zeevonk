use std::path::Path;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};

use zeevonk::theymx::Multiverse;

use zeevonk::attr::Attribute;
use zeevonk::project::file::ProjectFile;
use zeevonk::value::{AttributeValues, ClampedValue};

mod common;

criterion_main!(benches);
criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = bench_realistic_single_universe
}

pub fn bench_realistic_single_universe(c: &mut Criterion) {
    let project = zeevonk::project_builder::from_file(
        ProjectFile::load_from_folder(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("benches/data/projects/realistic_single_universe"),
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
