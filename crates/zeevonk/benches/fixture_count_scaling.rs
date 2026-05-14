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
    targets = bench_fixture_count_scaling
}

pub fn bench_fixture_count_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("Scaling Fixture Count");

    let counts = [50, 500, 1000];

    for count in counts {
        let path = "benches/data/gdtf_files/Generic@Dimmer@8_and_16bit_Generic_Dimmer.gdtf";
        let project = generate_project(
            count,
            path,
            "B4DAFF6B-3E52-451B-AFDB-E6C94C64F85D",
            "Default",
            "Dimmer",
        )
        .unwrap();
        let values =
            build_values(project.stage(), &[(AttributeName::Dimmer, ClampedValue::new(0.5))]);

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
        let path = "benches/data/gdtf_files/Generic@Dimmer@8_and_16bit_Generic_Dimmer.gdtf";
        let project = generate_project(
            count,
            path,
            "B4DAFF6B-3E52-451B-AFDB-E6C94C64F85D",
            "Default",
            "16 Bit",
        )
        .unwrap();
        let values =
            build_values(project.stage(), &[(AttributeName::Dimmer, ClampedValue::new(0.5))]);

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
        let path = "benches/data/gdtf_files/Generic@RGBW8@added_white_channel.gdtf";
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
                (AttributeName::ColorAddR, ClampedValue::new(0.5)),
                (AttributeName::ColorAddG, ClampedValue::new(0.5)),
                (AttributeName::ColorAddB, ClampedValue::new(0.5)),
                (AttributeName::ColorAddW, ClampedValue::new(0.5)),
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
