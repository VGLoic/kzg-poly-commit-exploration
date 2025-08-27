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

fn bench_evaluation_verification(c: &mut Criterion) {
    let mut group = c.benchmark_group("evaluation_verification");

    // Test with different polynomial degrees
    let degrees = [1, 100, 500, 1_000, 2_500];

    for degree in degrees.iter() {
        // Fix input point
        let input_point = generate_input_point(*degree);
        // Setup: Generate all required artifacts for each iteration
        let polynomial = generate_polynomial(*degree);
        let setup_artifacts = generate_setup_artifacts(*degree);

        // Generate commitment, evaluation, and proof
        let commitment = polynomial.commit(&setup_artifacts).unwrap();
        let evaluation = polynomial.evaluate(input_point.clone()).unwrap();
        let proof = evaluation
            .generate_proof(&polynomial, &setup_artifacts)
            .unwrap();

        group.bench_with_input(
            BenchmarkId::new("verify_proof", degree),
            &(&evaluation, &proof, &commitment, &setup_artifacts),
            |b, (eval, proof, commit, artifacts)| {
                b.iter(|| {
                    // Benchmark: Verify proof
                    let _is_valid = eval.verify_proof(proof, commit, artifacts).unwrap();
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_evaluation_verification);
criterion_main!(benches);
