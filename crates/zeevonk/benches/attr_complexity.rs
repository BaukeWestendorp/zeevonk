use criterion::BatchSize;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use rigger::gdtf::attr::AttributeName;
use zeevonk::theymx::Multiverse;
use zeevonk::value::ClampedValue;

use crate::common::{build_values, generate_project};

mod common;

criterion_main!(benches);
criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = bench_attribute_complexity
}

pub fn bench_attribute_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("Scaling Complexity");

    let fixture_count = 100;

    #[rustfmt::skip]
    let complexities = {
        use rigger::gdtf::attr::AttributeName as A;
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
        let path = "benches/data/gdtf_files/Zeevonk@Benchmark@Added_30ch_Mode.gdtf";
        let project = generate_project(
            fixture_count,
            path,
            "9125433D-A327-434E-95C5-D116FCBFB7D9",
            "30ch",
            "Bench",
        )
        .unwrap();

        let attr_values: Vec<(AttributeName, ClampedValue)> =
            attrs.iter().map(|a| (a.clone(), ClampedValue::new(0.5))).collect();

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
