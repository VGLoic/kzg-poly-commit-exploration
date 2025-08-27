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

fn generate_input_point(degree: u32) -> Scalar {
    Scalar::from(5).pow(degree as usize).add(&Scalar::from(20))
}

fn bench_polynomial_evaluation_and_proof(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluation_proof");
    group
        .measurement_time(Duration::from_secs_f32(25.0))
        .sample_size(50);

    // Test with different polynomial degrees
    let degrees = [1, 100, 500, 1_000, 2_500];

    for degree in degrees.iter() {
        // Fix input point
        let input_point = generate_input_point(*degree);

        let polynomial = generate_polynomial(*degree);
        // Setup: Generate polynomial, setup artifacts, evaluation point and evaluation
        let setup_artifacts = generate_setup_artifacts(*degree);
        let evaluation = polynomial.evaluate(input_point.clone()).unwrap();
        // Benchmark proof generation
        group.bench_with_input(
            BenchmarkId::new("proof_generation", degree),
            &(&polynomial, &evaluation, &setup_artifacts),
            |b, (p, eval, artifacts)| {
                b.iter(|| {
                    // Benchmark: Generate proof only
                    let _proof = eval.generate_proof(p, artifacts).unwrap();
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_polynomial_evaluation_and_proof);
criterion_main!(benches);
