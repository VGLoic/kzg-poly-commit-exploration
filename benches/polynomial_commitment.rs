use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use fake::{Fake, Faker};
use kzg_poly_commit_exploration::{
    polynomial::Polynomial,
    trusted_setup::{SetupArtifact, SetupArtifactsGenerator},
};
use rand::RngCore;

fn generate_polynomial(degree: u32) -> Polynomial {
    let mut coefficients: Vec<i128> = vec![];
    for _ in 0..(degree + 1) {
        coefficients.push(Faker.fake());
    }
    Polynomial::try_from(coefficients).unwrap()
}

fn generate_setup_artifacts(degree: u32) -> Vec<SetupArtifact> {
    let mut s_bytes = [0; 32]; // Secret is a 256-bit scalar
    rand::rng().fill_bytes(&mut s_bytes);
    SetupArtifactsGenerator::new(s_bytes)
        .take((degree + 1) as usize)
        .collect()
}

fn bench_polynomial_commitment(c: &mut Criterion) {
    let mut group = c.benchmark_group("polynomial_commitment");

    // Test with different polynomial degrees as specified
    let degrees = [1, 100, 500, 1_000, 2_500];

    for degree in degrees.iter() {
        group.bench_with_input(BenchmarkId::new("commit", degree), degree, |b, &degree| {
            b.iter_batched(
                || {
                    // Setup: Generate polynomial and setup artifacts for each iteration
                    let polynomial = generate_polynomial(degree);
                    let setup_artifacts = generate_setup_artifacts(degree);
                    (polynomial, setup_artifacts)
                },
                |(polynomial, setup_artifacts)| {
                    // Benchmark: Commit to polynomial
                    let _commitment = polynomial.commit(&setup_artifacts).unwrap();
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group!(benches, bench_polynomial_commitment);
criterion_main!(benches);
