use std::time::Duration;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use kzg_poly_commit_exploration::{
    polynomial::Polynomial,
    scalar::Scalar,
};

fn generate_polynomial(degree: u32) -> Polynomial {
    let coefficients: Vec<Scalar> = (0..(degree + 1))
        .map(|i| Scalar::from(5).pow(i as usize).add(&Scalar::from(10)))
        .collect();
    Polynomial::try_from(coefficients).unwrap()
}

fn generate_input_point(degree: u32) -> Scalar {
    Scalar::from(5).pow(degree as usize).add(&Scalar::from(20))
}

fn bench_polynomial_evaluation_and_proof(c: &mut Criterion) {
    let mut group = c.benchmark_group("polynomial_evaluation_and_proof");
    group.measurement_time(Duration::from_secs_f32(25.0)).sample_size(50);

    // Test with different polynomial degrees
    let degrees = [1, 100, 500, 1_000, 2_500];

    for degree in degrees.iter() {
        // Fix input point
        let input_point = generate_input_point(*degree);

        let polynomial = generate_polynomial(*degree);
        // Benchmark polynomial evaluation
        group.bench_with_input(
            BenchmarkId::new("evaluate", degree),
            &(&polynomial, input_point.clone()),
            |b, (p, input)| {
                b.iter(|| {
                    // Benchmark: Evaluate polynomial
                    let _evaluation = p.evaluate(input.clone()).unwrap();
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_polynomial_evaluation_and_proof);
criterion_main!(benches);
