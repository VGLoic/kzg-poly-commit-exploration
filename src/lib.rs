pub mod curves;
pub mod polynomial;
pub mod scalar;
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
        let proof = evaluation
            .generate_proof(polynomial, setup_artifacts)
            .unwrap();
        assert!(
            evaluation
                .verify_proof(&proof, &commitment, setup_artifacts)
                .unwrap(),
            "Verification of the proof fails for polynomial {polynomial} evaluated at point x = {input_point}",
        );
    }

    fn generate_polynomial(degree: u32) -> Polynomial {
        let mut coefficients: Vec<i128> = vec![];
        for _ in 0..(degree + 1) {
            coefficients.push(Faker.fake());
        }
        Polynomial::try_from(coefficients.as_slice()).unwrap()
    }

    fn generate_setup_artifacts(degree: u32) -> Vec<SetupArtifact> {
        let mut s_bytes = [0; 32]; // Secret is a 256-bit scalar
        rand::rng().fill_bytes(&mut s_bytes);
        SetupArtifactsGenerator::new(s_bytes)
            .take((degree + 1) as usize)
            .collect()
    }

    // TODO: Testing indicates some limitations on the possible values for the coefficients and the input points. There is a need to fix this and increase the coverage of these tests.
    #[test]
    fn test_kate_proof_for_polynomial_degree_one_over_multiple_input() {
        let setup_artifacts = &generate_setup_artifacts(1);
        for _ in 0..10 {
            let polynomial = generate_polynomial(1);

            for _ in 0..10 {
                let input_point: i128 = Faker.fake();
                run_kate_proof_test(&polynomial, input_point, setup_artifacts);
            }
        }
    }

    #[test]
    fn test_kate_proof_for_polynomial_degree_two_over_multiple_input() {
        let setup_artifacts = &generate_setup_artifacts(2);
        for _ in 0..10 {
            let polynomial = generate_polynomial(2);

            for _ in 0..10 {
                let input_point: i128 = Faker.fake();
                run_kate_proof_test(&polynomial, input_point, setup_artifacts);
            }
        }
    }

    #[test]
    fn test_kate_proof_over_multiple_degree_with_fixed_input() {
        let input_point: i128 = Faker.fake();

        for _ in 0..10 {
            let degree: u8 = Faker.fake();
            if degree == 0 {
                continue;
            }
            let polynomial = generate_polynomial(degree as u32);

            run_kate_proof_test(
                &polynomial,
                input_point,
                &generate_setup_artifacts(degree as u32),
            );
        }
    }
}
