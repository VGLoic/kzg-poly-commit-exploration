pub mod curves;
pub mod polynomial;
pub mod trusted_setup;

#[cfg(test)]
mod tests {
    use crate::polynomial::Polynomial;
    use crate::trusted_setup::SetupArtifactsGenerator;
    use rand::RngCore;

    #[test]
    fn test_commitment_for_polynomial_degree_one() {
        let mut s_bytes = [0; 48]; // Field elements are encoded in big endian form with 48 bytes
        rand::rng().fill_bytes(&mut s_bytes);
        let setup_artifacts: Vec<_> = SetupArtifactsGenerator::new(s_bytes).take(2).collect();

        // Polynomial to commit is `p(x) = 5x + 10
        let polynomial = Polynomial::try_from([10, 5].as_slice()).unwrap();
        let commitment = polynomial.commit(&setup_artifacts).unwrap();

        // We evaluate the polynomial at z = 1: `p(z) = y = p(1) = 15`
        // Quotient polynomial: `q(x) = (p(x) - y) / (x - z) = (5x - 5) / (x - 1) = 5`
        let evaluation = polynomial.evaluate(&1).unwrap();
        let proof = polynomial
            .generates_evaluation_proof(&evaluation, &setup_artifacts)
            .unwrap();
        assert!(
            evaluation
                .verify_proof(&proof, &commitment, &setup_artifacts)
                .unwrap()
        );
    }

    #[test]
    fn test_commitment_for_polynomial_degree_two() {
        let mut s_bytes = [0; 48]; // Field elements are encoded in big endian form with 48 bytes
        rand::rng().fill_bytes(&mut s_bytes);
        let setup_artifacts: Vec<_> = SetupArtifactsGenerator::new(s_bytes).take(3).collect();

        // Polynomial to commit is `p(x) = 2x^2 + 3x + 4`
        let polynomial = Polynomial::try_from([4, 3, 2].as_slice()).unwrap();
        let commitment = polynomial.commit(&setup_artifacts).unwrap();

        // We evaluate the polynomial at z = 2: `p(z) = y = p(2) = 8 + 6 + 4 = 18`
        // Quotient polynomial: `q(x) = (p(x) - y) / (x - z) = (2x^2 + 3x - 14) / (x - 2) = (x - 2) * (2x + 7) / (x - 2) = 2x + 7`
        let evaluation = polynomial.evaluate(&2).unwrap();
        let proof = polynomial
            .generates_evaluation_proof(&evaluation, &setup_artifacts)
            .unwrap();
        assert!(
            evaluation
                .verify_proof(&proof, &commitment, &setup_artifacts)
                .unwrap()
        );
    }

    #[test]
    fn test_commitment_for_polynomial_degree_two_with_negative_coefficients() {
        let mut s_bytes = [0; 48]; // Field elements are encoded in big endian form with 48 bytes
        rand::rng().fill_bytes(&mut s_bytes);
        let setup_artifacts: Vec<_> = SetupArtifactsGenerator::new(s_bytes).take(3).collect();

        // Polynomial to commit is `p(x) = 2x^2 - 3x - 1`
        let polynomial = Polynomial::try_from([-1, -3, 2].as_slice()).unwrap();
        let commitment = polynomial.commit(&setup_artifacts).unwrap();

        // We evaluate the polynomial at z = 2: `p(z) = y = p(2) = 8 - 6 - 1 = 1`
        // Quotient polynomial: `q(x) = (p(x) - y) / (x - z) = (2x^2 - 3x - 2) / (x - 2) = (x - 2) * (2x + 1) / (x - 2) = 2x + 1`
        let evaluation = polynomial.evaluate(&2).unwrap();
        let proof = polynomial
            .generates_evaluation_proof(&evaluation, &setup_artifacts)
            .unwrap();

        assert!(
            evaluation
                .verify_proof(&proof, &commitment, &setup_artifacts)
                .unwrap()
        );
    }
}
