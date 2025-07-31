use serde::{Deserialize, Serialize};

use crate::curves::{G2Point, bilinear_map};

use super::curves::G1Point;
use super::trusted_setup::SetupArtifact;

#[derive(Debug, Deserialize, Serialize)]
pub struct Polynomial {
    coefficients: Vec<i128>,
}

impl TryFrom<&[i128]> for Polynomial {
    type Error = anyhow::Error;

    fn try_from(value: &[i128]) -> Result<Self, Self::Error> {
        if value.len() > u32::MAX as usize {
            return Err(anyhow::anyhow!(
                "Too many coefficients for polynomial, only 2**32 - 1 coefficients is supported. Got {}",
                value.len()
            ));
        }

        let mut coefficients = value.to_vec();

        while let Some(last_value) = coefficients.last()
            && *last_value == 0
        {
            coefficients.pop();
        }

        Ok(Polynomial { coefficients })
    }
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub struct Evaluation {
    pub point: i128,
    pub result: i128,
}

impl Evaluation {
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
            &setup_artifacts[1].g2.sub(&G2Point::from_i128(self.point)),
        );
        let rhs = bilinear_map(
            &commitment.sub(&G1Point::from_i128(self.result)),
            &G2Point::from_i128(1),
        );

        Ok(lhs == rhs)
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
    pub fn from_constant(a: i128) -> Polynomial {
        Polynomial {
            coefficients: vec![a],
        }
    }

    /// Evaluate the polynomial at an input point
    ///
    /// * `x` - Input point
    pub fn evaluate(&self, x: &i128) -> Result<Evaluation, anyhow::Error> {
        let mut evaluation: i128 = 0;
        for (degree, coefficient) in self.coefficients.iter().enumerate() {
            let x_powered = x.checked_pow(degree as u32).ok_or(anyhow::anyhow!(
                "[evaluate] Overflow while pow {x}^{degree}"
            ))?;
            let contribution = coefficient.checked_mul(x_powered).ok_or(anyhow::anyhow!(
                "[evaluate] Overflow while {coefficient} * {x_powered}"
            ))?;
            evaluation = evaluation.checked_add(contribution).ok_or(anyhow::anyhow!(
                "[evaluate] Overflow while {evaluation} + {contribution}"
            ))?;
        }
        Ok(Evaluation {
            point: *x,
            result: evaluation,
        })
    }

    /// Generates a Kate proof for a given evaluation
    ///
    /// * `evaluation` - The evaluation for which the proof is generated
    /// * `setup_artifacts` - List of setup artifacts for both elliptic curve groups. There must at least `degree` artifacts.
    pub fn generate_evaluation_proof(
        &self,
        evaluation: &Evaluation,
        setup_artifacts: &[SetupArtifact],
    ) -> Result<G1Point, anyhow::Error> {
        self.sub(&Polynomial::from_constant(evaluation.result))?
            .divide_by_root(&evaluation.point)?
            .commit(setup_artifacts)
    }

    /// Subtract a polynomial from the current one
    ///
    /// * `p` - Polynomial to subtract from the current one
    pub fn sub(&self, p: &Polynomial) -> Result<Polynomial, anyhow::Error> {
        let a_length = self.coefficients.len();
        let b_length = p.coefficients.len();

        let mut coefficients: Vec<i128>;
        if a_length > b_length {
            coefficients = self.coefficients.clone();
            for (i, rhs) in p.coefficients.iter().enumerate() {
                coefficients[i] = coefficients[i].checked_sub(*rhs).ok_or(anyhow::anyhow!(
                    "[sub] Overflow while {} - {rhs}",
                    coefficients[i]
                ))?;
            }
        } else {
            coefficients = p
                .coefficients
                .iter()
                .map(|x| {
                    x.checked_neg()
                        .ok_or(anyhow::anyhow!("[sub] Overflow while negating {x}"))
                })
                .collect::<Result<Vec<_>, anyhow::Error>>()?;
            for (i, lhs) in self.coefficients.iter().enumerate() {
                coefficients[i] = lhs.checked_add(coefficients[i]).ok_or(anyhow::anyhow!(
                    "[sub] Overflow while {lhs} - {}",
                    coefficients[i]
                ))?;
            }
        }

        Polynomial::try_from(coefficients.as_slice())
    }

    /// Divides the polynomial by the divider polynomial `x - root` and returns the quotient polynomial.
    ///
    /// * `root` - Root of the polynomial
    pub fn divide_by_root(&self, root: &i128) -> Result<Polynomial, anyhow::Error> {
        let higher_order_coefficient = match self.coefficients.last() {
            None => {
                return Ok(Polynomial {
                    coefficients: vec![],
                });
            }
            Some(v) => v,
        };
        if self.coefficients.len() == 1 {
            if *higher_order_coefficient == 0 {
                return Ok(Polynomial {
                    coefficients: vec![],
                });
            } else {
                return Err(anyhow::anyhow!("Unable to divide a constant polynomial"));
            }
        }
        let mut quotient_coefficients_reversed = vec![*higher_order_coefficient];
        // We skip the higher degree as it is handled at initialisation, and we skip the degree zero as it is checked at the end
        let mut last_coefficient_found = *higher_order_coefficient;
        for i in (1..self.coefficients.len() - 1).rev() {
            let coefficient = self.coefficients[i];
            let contribution_from_root =
                root.checked_mul(last_coefficient_found)
                    .ok_or(anyhow::anyhow!(
                        "[divide_by_root] Overflow while {root} * {last_coefficient_found}"
                    ))?;
            last_coefficient_found =
                coefficient
                    .checked_add(contribution_from_root)
                    .ok_or(anyhow::anyhow!(
                        "[divide_by_root] Overflow while {coefficient} + {contribution_from_root}"
                    ))?;

            quotient_coefficients_reversed.push(last_coefficient_found);
        }

        quotient_coefficients_reversed.reverse();

        // We check that the constant term is correct: -1 * root * constant term of q = constant term of p
        let rebuilt_constant_term = root
            .checked_mul(quotient_coefficients_reversed[0])
            .ok_or(anyhow::anyhow!(
                "[divide_by_root] Overflow while {root} * {}",
                quotient_coefficients_reversed[0]
            ))?
            .checked_neg()
            .ok_or(anyhow::anyhow!(
                "[divide_by_root] Overflow while taking the negative of rebuilt constant term"
            ))?;
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
            let contribution = setup_point.mult(*coefficient);
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

        let higher_degree_coefficient = self.coefficients[self.coefficients.len() - 1];
        let mut displayed = format!(
            "{}{}",
            if higher_degree_coefficient < 0 {
                "-"
            } else {
                ""
            },
            display_non_zero_coefficient(higher_degree_coefficient, self.coefficients.len() - 1)
        );

        for i in (0..(self.coefficients.len() - 1)).rev() {
            let c = self.coefficients[i];
            if c == 0 {
                continue;
            }
            displayed += format!(
                " {} {}",
                if c > 0 { "+" } else { "-" },
                display_non_zero_coefficient(c, i)
            )
            .as_str();
        }

        write!(f, "{displayed}")
    }
}

fn display_non_zero_coefficient(c: i128, degree: usize) -> String {
    let degree_string = match degree {
        0 => "".to_owned(),
        1 => "x".to_owned(),
        other => format!("x^{other}"),
    };
    if degree > 0 && (c == 1 || c == -1) {
        return degree_string;
    }
    format!("{}{degree_string}", c.abs())
}
