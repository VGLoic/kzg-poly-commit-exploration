use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use fake::{Fake, Faker};
use kzg_poly_commit_exploration::{
    polynomial::Polynomial,
    scalar::Scalar,
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

fn bench_polynomial_evaluation_and_proof(c: &mut Criterion) {
    let mut group = c.benchmark_group("polynomial_evaluation_and_proof");

    // Test with different polynomial degrees
    let degrees = [1, 5, 10, 25, 50];

    for degree in degrees.iter() {
        // Benchmark polynomial evaluation
        group.bench_with_input(
            BenchmarkId::new("evaluate", degree),
            degree,
            |b, &degree| {
                b.iter_batched(
                    || {
                        // Setup: Generate polynomial and evaluation point for each iteration
                        let polynomial = generate_polynomial(degree);
                        let input_point = Scalar::from_i128(Faker.fake::<i128>());
                        (polynomial, input_point)
                    },
                    |(polynomial, input_point)| {
                        // Benchmark: Evaluate polynomial
                        let _evaluation = polynomial.evaluate(input_point).unwrap();
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        // Benchmark proof generation
        group.bench_with_input(
            BenchmarkId::new("proof_generation", degree),
            degree,
            |b, &degree| {
                b.iter_batched(
                    || {
                        // Setup: Generate polynomial, setup artifacts, evaluation point, and evaluation for each iteration
                        let polynomial = generate_polynomial(degree);
                        let setup_artifacts = generate_setup_artifacts(degree);
                        let input_point = Scalar::from_i128(Faker.fake::<i128>());
                        let evaluation = polynomial.evaluate(input_point).unwrap();
                        (polynomial, setup_artifacts, evaluation)
                    },
                    |(polynomial, setup_artifacts, evaluation)| {
                        // Benchmark: Generate proof only
                        let _proof = evaluation
                            .generate_proof(&polynomial, &setup_artifacts)
                            .unwrap();
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_polynomial_evaluation_and_proof);
criterion_main!(benches);
