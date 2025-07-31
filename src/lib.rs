pub mod curves;
pub mod polynomial;
pub mod trusted_setup;

#[cfg(test)]
mod tests {
    use crate::trusted_setup::SetupArtifactsGenerator;
    use crate::{polynomial::Polynomial, trusted_setup::SetupArtifact};
    use fake::{Fake, Faker};
    use rand::RngCore;

    fn run_kate_proof_test(
        polynomial: &Polynomial,
        input_point: i128,
        setup_artifacts: &[SetupArtifact],
    ) {
        let commitment = polynomial.commit(setup_artifacts).unwrap();

        let evaluation = polynomial.evaluate(&input_point).unwrap();
        let proof = polynomial
            .generates_evaluation_proof(&evaluation, setup_artifacts)
            .unwrap();
        assert!(
            evaluation
                .verify_proof(&proof, &commitment, setup_artifacts)
                .unwrap(),
            "Verification of the proof fails for polynomial {polynomial} evaluated at point x = {input_point}",
        );
    }

    fn generate_polynomial(degree: u32) -> Polynomial {
        let mut coefficients: Vec<i32> = vec![];
            for _ in 0..(degree + 1) {
                coefficients.push(Faker.fake());
            }
            Polynomial::try_from(
                coefficients
                    .into_iter()
                    .map(i128::from)
                    .collect::<Vec<i128>>()
                    .as_slice(),
            )
            .unwrap()
    }

    fn generate_setup_artifacts(degree: u32) -> Vec<SetupArtifact> {
        let mut s_bytes = [0; 48]; // Field elements are encoded in big endian form with 48 bytes
        rand::rng().fill_bytes(&mut s_bytes);
        SetupArtifactsGenerator::new(s_bytes)
            .take((degree + 1) as usize)
            .collect()
    }

    #[test]
    fn test_kate_proof_for_polynomial_degree_one_over_multiple_input() {
        let setup_artifacts = &generate_setup_artifacts(1);
        for _ in 0..10 {
            let polynomial = generate_polynomial(1);

            for _ in 0..10 {
                let input_point: i32 = Faker.fake();
                run_kate_proof_test(&polynomial, input_point.into(), setup_artifacts);
            }
        }
    }

    #[test]
    fn test_kate_proof_for_polynomial_degree_two_over_multiple_input() {
        let setup_artifacts = &generate_setup_artifacts(2);
        for _ in 0..10 {
            let polynomial = generate_polynomial(2);

            for _ in 0..10 {
                let input_point: i32 = Faker.fake();
                run_kate_proof_test(&polynomial, input_point.into(), setup_artifacts);
            }
        }
    }

    #[test]
    fn test_kate_proof_over_multiple_degree_with_fixed_input() {
        for _ in 0..10 {
            let degree: u8 = Faker.fake();
            if degree == 0 {
                continue;
            }
            let polynomial = generate_polynomial(degree as u32);

            let input_point: u8 = 1;

            run_kate_proof_test(
                &polynomial,
                input_point.into(),
                &generate_setup_artifacts(degree as u32),
            );
        }
    }
}
