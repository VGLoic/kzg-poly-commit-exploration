use serde::{Deserialize, Serialize};

use super::{
    curves::{G1Point, G2Point, bilinear_map},
    scalar::Scalar,
    trusted_setup::SetupArtifact,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Polynomial {
    coefficients: Vec<Scalar>,
}

impl TryFrom<&[i128]> for Polynomial {
    type Error = anyhow::Error;

    fn try_from(value: &[i128]) -> Result<Self, Self::Error> {
        Self::try_from(
            value
                .iter()
                .map(|a| Scalar::from_i128(*a))
                .collect::<Vec<Scalar>>()
                .as_slice(),
        )
    }
}

impl TryFrom<&[Scalar]> for Polynomial {
    type Error = anyhow::Error;

    fn try_from(value: &[Scalar]) -> Result<Self, Self::Error> {
        if value.len() > u32::MAX as usize {
            return Err(anyhow::anyhow!(
                "Too many coefficients for polynomial, only 2**32 - 1 coefficients is supported. Got {}",
                value.len()
            ));
        }

        let mut coefficients: Vec<Scalar> = vec![];
        let mut is_empty = true;
        for v in value.iter().rev() {
            if is_empty {
                if v.is_zero() {
                    continue;
                } else {
                    is_empty = false;
                }
            }
            coefficients.push(v.clone());
        }
        coefficients.reverse();

        Ok(Polynomial { coefficients })
    }
}

impl From<Scalar> for Polynomial {
    fn from(value: Scalar) -> Self {
        let mut coefficients = vec![];
        if !value.is_zero() {
            coefficients.push(value);
        }
        Polynomial { coefficients }
    }
}

impl Polynomial {
    /// Return the degree of the polynomial.
    ///
    /// Degree is derived as one minus the number of coefficients.
    pub fn degree(&self) -> u32 {
        if self.coefficients.is_empty() {
            return 0;
        }
        (self.coefficients.len() - 1) as u32
    }

    /// Creates a polynomial of order 0 from a constant
    ///
    /// * `a` - Constant
    pub fn from_constant(a: Scalar) -> Self {
        Self::from(a)
    }

    /// Evaluate the polynomial at an input point
    ///
    /// * `x` - Input point
    pub fn evaluate(&self, x: &i128) -> Result<Evaluation, anyhow::Error> {
        let mut evaluation = Scalar::from_i128(0);
        let x_scalar = Scalar::from_i128(*x);
        for (degree, coefficient) in self.coefficients.iter().enumerate() {
            let x_powered = x_scalar.pow(degree);
            let contribution = coefficient.mul(&x_powered);
            evaluation = evaluation.add(&contribution);
        }
        Ok(Evaluation {
            point: x_scalar,
            result: evaluation,
        })
    }

    /// Subtract a polynomial from the current one
    ///
    /// * `p` - Polynomial to subtract from the current one
    pub fn sub(&self, p: &Self) -> Result<Self, anyhow::Error> {
        let a_length = self.coefficients.len();
        let b_length = p.coefficients.len();

        let mut coefficients: Vec<Scalar>;
        if a_length > b_length {
            coefficients = self.coefficients.clone();
            for (i, rhs) in p.coefficients.iter().enumerate() {
                coefficients[i] = coefficients[i].sub(rhs);
            }
        } else {
            coefficients = p.coefficients.iter().map(|x| x.neg()).collect();
            for (i, lhs) in self.coefficients.iter().enumerate() {
                coefficients[i] = lhs.add(&coefficients[i]);
            }
        }
        Polynomial::try_from(coefficients.as_slice())
    }

    /// Divides the polynomial by the divider polynomial `x - root` and returns the quotient polynomial.
    ///
    /// * `root` - Root of the polynomial
    pub fn divide_by_root(&self, root: &Scalar) -> Result<Self, anyhow::Error> {
        let higher_order_coefficient = match self.coefficients.last() {
            None => {
                return Ok(Polynomial {
                    coefficients: vec![],
                });
            }
            Some(v) => v.clone(),
        };
        if self.coefficients.len() == 1 {
            if higher_order_coefficient.is_zero() {
                return Ok(Polynomial {
                    coefficients: vec![],
                });
            } else {
                return Err(anyhow::anyhow!("Unable to divide a constant polynomial"));
            }
        }
        // REMIND ME
        let mut quotient_coefficients_reversed = vec![higher_order_coefficient.clone()];
        // We skip the higher degree as it is handled at initialisation, and we skip the degree zero as it is checked at the end
        let mut last_coefficient_found = higher_order_coefficient;
        for i in (1..self.coefficients.len() - 1).rev() {
            let coefficient = &self.coefficients[i];
            let contribution_from_root = root.mul(&last_coefficient_found);
            last_coefficient_found = coefficient.add(&contribution_from_root);

            quotient_coefficients_reversed.push(last_coefficient_found.clone());
        }

        quotient_coefficients_reversed.reverse();

        // We check that the constant term is correct: -1 * root * constant term of q = constant term of p
        let rebuilt_constant_term = root.mul(&quotient_coefficients_reversed[0]).neg();

        println!("rebuilt_constant_term: {rebuilt_constant_term}");
        if rebuilt_constant_term != self.coefficients[0] {
            return Err(anyhow::anyhow!(
                "[divide_by_root] Fail to divide the polynomial by a root, constant terms do not add up"
            ));
        }

        Polynomial::try_from(quotient_coefficients_reversed.as_slice())
    }

    /// Generate the G1Point representing the commit to the polynomial using setup artifacts.
    ///
    /// * `setup_artifacts` - List of setup artifacts for both elliptic curve groups. There must at least `degree + 1` artifacts.
    pub fn commit(&self, setup_artifacts: &[SetupArtifact]) -> Result<G1Point, anyhow::Error> {
        if (self.degree() + 1) as usize > setup_artifacts.len() {
            return Err(anyhow::anyhow!(
                "Setup does not allow for commitment generation of the polynomial. The polynomial degree is too high."
            ));
        }

        let mut commitment = G1Point::from_i128(0);
        for (i, coefficient) in self.coefficients.iter().enumerate() {
            let setup_point = &setup_artifacts[i].g1;
            let contribution = setup_point.mult(coefficient);
            commitment = commitment.add(&contribution);
        }

        Ok(commitment)
    }
}

impl std::fmt::Display for Polynomial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.coefficients.is_empty() {
            return write!(f, "0");
        }

        let higher_degree_coefficient = &self.coefficients[self.coefficients.len() - 1];
        let mut displayed =
            display_non_zero_coefficient(higher_degree_coefficient, self.coefficients.len() - 1);

        for i in (0..(self.coefficients.len() - 1)).rev() {
            let c = &self.coefficients[i];
            if c.is_zero() {
                continue;
            }
            displayed += format!(" + {}", display_non_zero_coefficient(c, i)).as_str();
        }

        write!(f, "{displayed}")
    }
}

fn display_non_zero_coefficient(c: &Scalar, degree: usize) -> String {
    let degree_string = match degree {
        0 => "".to_owned(),
        1 => "x".to_owned(),
        other => format!("x^{other}"),
    };
    format!("{c}{degree_string}")
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Evaluation {
    pub point: Scalar,
    pub result: Scalar,
}

impl Evaluation {
    /// Generates a Kate proof for a given evaluation
    ///
    /// * `polynomial` - The polynomial associated with the evaluation
    /// * `setup_artifacts` - List of setup artifacts for both elliptic curve groups. There must at least `degree` artifacts.
    pub fn generate_proof(
        &self,
        polynomial: &Polynomial,
        setup_artifacts: &[SetupArtifact],
    ) -> Result<G1Point, anyhow::Error> {
        polynomial
            .sub(&Polynomial::from_constant(self.result.clone()))?
            .divide_by_root(&self.point)?
            .commit(setup_artifacts)
    }

    /// Verify the Kate proof given a proof, a commitment and the setup artifacts
    ///
    /// * `proof` - Evaluation proof
    /// * `commitment` - Commitment of the underlying polynomial
    /// * `setup_artifacts` - List of setup artifacts for both elliptic curve groups. There must at least 2 artifacts.
    pub fn verify_proof(
        &self,
        proof: &G1Point,
        commitment: &G1Point,
        setup_artifacts: &[SetupArtifact],
    ) -> Result<bool, anyhow::Error> {
        let lhs = bilinear_map(
            proof,
            &setup_artifacts[1]
                .g2
                .sub(&G2Point::from_scalar(self.point.clone())),
        );
        let rhs = bilinear_map(
            &commitment.sub(&G1Point::from_scalar(self.result.clone())),
            &G2Point::from_i128(1),
        );

        Ok(lhs == rhs)
    }
}
