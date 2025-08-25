use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use kzg_poly_commit_exploration::trusted_setup::SetupArtifactsGenerator;
use rand::RngCore;

fn bench_trusted_setup_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("trusted_setup_generation");
    
    // Test with different polynomial degrees as specified
    let degrees = [2, 20, 2_000];
    
    for degree in degrees.iter() {
        group.bench_with_input(
            BenchmarkId::new("setup_generation", degree),
            degree,
            |b, &degree| {
                b.iter_batched(
                    || {
                        // Setup: Generate random secret for each iteration
                        let mut s_bytes = [0; 32];
                        rand::rng().fill_bytes(&mut s_bytes);
                        s_bytes
                    },
                    |s_bytes| {
                        // Benchmark: Generate setup artifacts
                        let _setup_artifacts: Vec<_> = SetupArtifactsGenerator::new(s_bytes)
                            .take((degree + 1) as usize)
                            .collect();
                    },
                    criterion::BatchSize::SmallInput
                );
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_trusted_setup_generation);
criterion_main!(benches);