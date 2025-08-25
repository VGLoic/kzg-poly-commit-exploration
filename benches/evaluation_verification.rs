use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use kzg_poly_commit_exploration::{
    polynomial::Polynomial,
    scalar::Scalar,
    trusted_setup::{SetupArtifact, SetupArtifactsGenerator},
};
use fake::{Fake, Faker};
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

fn bench_evaluation_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluation_verification");
    
    // Test with different polynomial degrees
    let degrees = [1, 5, 10, 25, 50];
    
    for degree in degrees.iter() {
        group.bench_with_input(
            BenchmarkId::new("verify_proof", degree),
            degree,
            |b, &degree| {
                b.iter_batched(
                    || {
                        // Setup: Generate all required artifacts for each iteration
                        let polynomial = generate_polynomial(degree);
                        let setup_artifacts = generate_setup_artifacts(degree);
                        let input_point = Scalar::from_i128(Faker.fake::<i128>());
                        
                        // Generate commitment, evaluation, and proof
                        let commitment = polynomial.commit(&setup_artifacts).unwrap();
                        let evaluation = polynomial.evaluate(input_point).unwrap();
                        let proof = evaluation.generate_proof(&polynomial, &setup_artifacts).unwrap();
                        
                        (evaluation, proof, commitment, setup_artifacts)
                    },
                    |(evaluation, proof, commitment, setup_artifacts)| {
                        // Benchmark: Verify proof
                        let _is_valid = evaluation.verify_proof(&proof, &commitment, &setup_artifacts).unwrap();
                    },
                    criterion::BatchSize::SmallInput
                );
            },
        );
    }
    
    group.finish();
}

criterion_group!(benches, bench_evaluation_verification);
criterion_main!(benches);