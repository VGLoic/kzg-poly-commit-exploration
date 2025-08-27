use std::time::Duration;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use kzg_poly_commit_exploration::{
    polynomial::Polynomial,
    scalar::Scalar,
    trusted_setup::{SetupArtifact, SetupArtifactsGenerator},
};

fn generate_polynomial(degree: u32) -> Polynomial {
    let coefficients: Vec<Scalar> = (0..(degree + 1))
        .map(|i| Scalar::from(5).pow(i as usize).add(&Scalar::from(10)))
        .collect();
    Polynomial::try_from(coefficients).unwrap()
}

fn generate_setup_artifacts(degree: u32) -> Vec<SetupArtifact> {
    let mut s_bytes = [0; 32]; // Secret is a 256-bit scalar
    s_bytes.copy_from_slice(&(0..32).collect::<Vec<u8>>());
    SetupArtifactsGenerator::new(s_bytes)
        .take((degree + 1) as usize)
        .collect()
}

fn bench_polynomial_commitment(c: &mut Criterion) {
    let mut group = c.benchmark_group("polynomial_commitment");
    group
        .measurement_time(Duration::from_secs_f32(20.0))
        .sample_size(75);

    // Test with different polynomial degrees as specified
    let degrees = [1, 100, 500, 1_000, 2_500];

    for degree in degrees.iter() {
        let polynomial = generate_polynomial(*degree);
        let setup_artifacts = generate_setup_artifacts(*degree);

        group.bench_with_input(
            BenchmarkId::new("commit", degree),
            &(&polynomial, &setup_artifacts),
            |b, (p, artifacts)| {
                b.iter(|| {
                    // Benchmark: Commit to polynomial
                    let _commitment = p.commit(artifacts).unwrap();
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_polynomial_commitment);
criterion_main!(benches);
