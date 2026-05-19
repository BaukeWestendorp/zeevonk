use std::path::Path;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};

use rigger::gdtf::attr::AttributeName;
use zeevonk::project::ProjectFile;
use zeevonk::theymx::Multiverse;
use zeevonk::value::{AttributeValues, ClampedValue};

mod common;

criterion_main!(benches);
criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = bench_realistic_single_universe
}

pub fn bench_realistic_single_universe(c: &mut Criterion) {
    let project = zeevonk::project::builder::from_file(
        ProjectFile::load_from_folder(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("benches/data/projects/realistic_single_universe"),
        )
        .unwrap(),
    )
    .unwrap();

    let mut values = AttributeValues::new();

    let mut set_all = |addr: String, attr: AttributeName| {
        values.set(addr.parse().unwrap(), attr, ClampedValue::new(0.5));
    };

    for n in 101..=104 {
        set_all(format!("{n}.1"), AttributeName::Pan);
        set_all(format!("{n}.1.1"), AttributeName::Color(1));
        set_all(format!("{n}.1.1"), AttributeName::Shutter(1));
        set_all(format!("{n}.1.1"), AttributeName::Dimmer);
        set_all(format!("{n}.1.1"), AttributeName::Gobo(1));
        set_all(format!("{n}.1.1"), AttributeName::Prism(1));
        set_all(format!("{n}.1.1"), AttributeName::PrismPos(1));
        set_all(format!("{n}.1.1"), AttributeName::EffectsPos(1));
        set_all(format!("{n}.1.1"), AttributeName::Frost(1));
        set_all(format!("{n}.1.1"), AttributeName::Focus(1));
        set_all(format!("{n}.1.1"), AttributeName::Tilt);
        set_all(format!("{n}.1.1"), AttributeName::PositionMSpeed);
        set_all(format!("{n}.1.1"), AttributeName::FixtureGlobalReset);
        set_all(format!("{n}.1.1"), AttributeName::LampControl);
    }

    for n in 201..=208 {
        set_all(format!("{n}"), AttributeName::PositionMSpeed);
        set_all(format!("{n}"), AttributeName::Control(1));
        set_all(format!("{n}.1"), AttributeName::Pan);
        set_all(format!("{n}.1.1"), AttributeName::Tilt);
        set_all(format!("{n}.1.1"), AttributeName::ColorAddR);
        set_all(format!("{n}.1.1"), AttributeName::ColorAddG);
        set_all(format!("{n}.1.1"), AttributeName::ColorAddB);
        set_all(format!("{n}.1.1"), AttributeName::ColorAddW);
        set_all(format!("{n}.1.1"), AttributeName::Cto);
        set_all(format!("{n}.1.1"), AttributeName::Color(1));
        set_all(format!("{n}.1.1"), AttributeName::Zoom);
        set_all(format!("{n}.1.1"), AttributeName::Shutter(1));
        set_all(format!("{n}.1.1"), AttributeName::Dimmer);
    }

    for n in 301..=312 {
        set_all(format!("{n}.1"), AttributeName::Pan);
        set_all(format!("{n}.1"), AttributeName::PanRotate);
        set_all(format!("{n}.1.1"), AttributeName::Tilt);
        set_all(format!("{n}.1.1"), AttributeName::TiltRotate);
        set_all(format!("{n}.1.1"), AttributeName::ColorAddR);
        set_all(format!("{n}.1.1"), AttributeName::ColorAddG);
        set_all(format!("{n}.1.1"), AttributeName::ColorAddB);
        set_all(format!("{n}.1.1"), AttributeName::Gobo(1));
        set_all(format!("{n}.1.1"), AttributeName::Prism(1));
        set_all(format!("{n}.1.1"), AttributeName::PrismPos(1));
        set_all(format!("{n}.1.1"), AttributeName::Prism(2));
        set_all(format!("{n}.1.1"), AttributeName::PrismPos(2));
        set_all(format!("{n}.1.1"), AttributeName::Shutter(1));
        set_all(format!("{n}.1.1"), AttributeName::Dimmer);
        set_all(format!("{n}.1.1"), AttributeName::Focus(1));
        set_all(format!("{n}.1.1"), AttributeName::Frost(1));
        set_all(format!("{n}.1.1"), AttributeName::DimmerMode);
        set_all(format!("{n}.1.1"), AttributeName::PositionMSpeed);
        set_all(format!("{n}.1.1"), AttributeName::Function);
    }

    for n in 401..=404 {
        set_all(format!("{n}.1"), AttributeName::Pan);
        set_all(format!("{n}.1"), AttributeName::PanRotate);
        set_all(format!("{n}.1.1"), AttributeName::Tilt);
        set_all(format!("{n}.1.1"), AttributeName::TiltRotate);
        set_all(format!("{n}.1.1"), AttributeName::ColorAddR);
        set_all(format!("{n}.1.1"), AttributeName::ColorAddG);
        set_all(format!("{n}.1.1"), AttributeName::ColorAddB);
        set_all(format!("{n}.1.1"), AttributeName::Gobo(1));
        set_all(format!("{n}.1.1"), AttributeName::Prism(1));
        set_all(format!("{n}.1.1"), AttributeName::PrismPos(1));
        set_all(format!("{n}.1.1"), AttributeName::Prism(2));
        set_all(format!("{n}.1.1"), AttributeName::PrismPos(2));
        set_all(format!("{n}.1.1"), AttributeName::Shutter(1));
        set_all(format!("{n}.1.1"), AttributeName::Dimmer);
        set_all(format!("{n}.1.1"), AttributeName::Focus(1));
        set_all(format!("{n}.1.1"), AttributeName::Frost(1));
        set_all(format!("{n}.1.1"), AttributeName::DimmerMode);
        set_all(format!("{n}.1.1"), AttributeName::PositionMSpeed);
        set_all(format!("{n}.1.1"), AttributeName::Function);
    }

    for n in 501..=505 {
        set_all(format!("{n}.1"), AttributeName::Tilt);
        set_all(format!("{n}.1"), AttributeName::Control(1));
        set_all(format!("{n}.1.1"), AttributeName::Dimmer);
        set_all(format!("{n}.1.1"), AttributeName::StrobeDuration);
        set_all(format!("{n}.1.1"), AttributeName::StrobeRate);
        set_all(format!("{n}.1.1"), AttributeName::StrobeModeStrobe);
        set_all(format!("{n}.1.2"), AttributeName::Dimmer);
        set_all(format!("{n}.1.2"), AttributeName::StrobeDuration);
        set_all(format!("{n}.1.2"), AttributeName::StrobeRate);
        set_all(format!("{n}.1.2"), AttributeName::StrobeModeStrobe);
        set_all(format!("{n}.1.2"), AttributeName::ColorAddR);
        set_all(format!("{n}.1.2"), AttributeName::ColorAddG);
        set_all(format!("{n}.1.2"), AttributeName::ColorAddB);
    }

    set_all("601.1".to_string(), AttributeName::Dimmer);

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
