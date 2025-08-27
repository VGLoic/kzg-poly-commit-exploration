use std::time::Duration;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use kzg_poly_commit_exploration::trusted_setup::SetupArtifactsGenerator;

fn bench_trusted_setup_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("trusted_setup_generation");
    group.measurement_time(Duration::from_secs_f32(25.0)).sample_size(75);

    // Test with different polynomial degrees as specified
    let degrees = [1, 100, 500, 1_000, 2_500];

    let mut s_bytes = [0; 32]; // Secret is a 256-bit scalar
    s_bytes.copy_from_slice(&(0..32).collect::<Vec<u8>>());

    for degree in degrees.iter() {
        group.bench_with_input(
            BenchmarkId::new("setup_generation", degree),
            degree,
            |b, &degree| {
                b.iter(|| {
                    // Benchmark: Generate setup artifacts
                    let _setup_artifacts: Vec<_> = SetupArtifactsGenerator::new(s_bytes)
                        .take((degree + 1) as usize)
                        .collect();
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_trusted_setup_generation);
criterion_main!(benches);
