use serde::{Deserialize, Serialize};

use crate::curves::G1Point;

use super::trusted_setup::SetupArtifact;

#[derive(Debug, Deserialize, Serialize)]
pub struct Polynomial {
    coefficients: Vec<i8>,
}

impl From<&[i8]> for Polynomial {
    fn from(value: &[i8]) -> Self {
        let mut coefficients = value.to_vec();

        while let Some(last_value) = coefficients.last()
            && *last_value == 0
        {
            coefficients.pop();
        }

        Polynomial { coefficients }
    }
}

impl Polynomial {
    /// Return the degree of the polynomial.
    ///
    /// Degree is derived as one plus the number of coefficients.
    pub fn degree(&self) -> usize {
        if self.coefficients.is_empty() {
            return 0;
        }
        self.coefficients.len() - 1
    }

    /// Evaluate the polynomial at an input point
    ///
    /// * `x` - Input point
    pub fn evaluate(&self, x: &i128) -> Result<i128, anyhow::Error> {
        if u32::try_from(self.coefficients.len()).is_err() {
            return Err(anyhow::anyhow!(
                "Too many coefficients for polynomial evaluation, only 2**32 coefficients is supported"
            ));
        }
        let mut evaluation: i128 = 0;
        for (power, coefficient) in (0_u32..).zip(self.coefficients.iter()) {
            let x_powered = x
                .checked_pow(power)
                .ok_or(anyhow::anyhow!("Overflow while pow {x}^{power}"))?;
            let contribution =
                i128::from(*coefficient)
                    .checked_mul(x_powered)
                    .ok_or(anyhow::anyhow!(
                        "Overflow while {coefficient} * {x_powered}"
                    ))?;
            evaluation = evaluation.checked_add(contribution).ok_or(anyhow::anyhow!(
                "Overflow while {evaluation} + {contribution}"
            ))?;
        }
        Ok(evaluation)
    }

    /// Subtract a polynomial from the current one
    ///
    /// * `p` - Polynomial to subract to the current one
    pub fn sub(&self, p: Polynomial) -> Polynomial {
        let a_length = self.coefficients.len();
        let b_length = p.coefficients.len();

        let mut coefficients: Vec<i8>;
        if a_length > b_length {
            coefficients = self.coefficients.clone();
            for (i, coeff) in p.coefficients.iter().enumerate() {
                coefficients[i] -= coeff;
            }
        } else {
            coefficients = p.coefficients.clone();
            for (i, coeff) in self.coefficients.iter().enumerate() {
                coefficients[i] = coeff - coefficients[i];
            }
        }

        Polynomial::from(coefficients.as_ref())
    }

    /// Divides the polynomial by the divider polynomial `x - root` and returns the quotient polynomial.
    ///
    /// * `root` - Root of the polynomial
    pub fn divide_by_root(&self, root: &i128) -> Result<Polynomial, anyhow::Error> {
        let higher_order_cofficient = self.coefficients.last().ok_or(anyhow::anyhow!(
            "Unable to divide a polynomial of degree zero"
        ))?;
        let mut quotient_coefficients_reversed = vec![*higher_order_cofficient];
        // We skip the higher degree as it is handled at initialisation, and we skip the degree zero as it is checked at the end
        let mut last_coefficient_found = *higher_order_cofficient;
        for coefficient in self.coefficients.iter().skip(1).rev().skip(1) {
            // REMIND ME
            let contribution_from_root = i8::try_from(*root).unwrap() * last_coefficient_found;
            last_coefficient_found = *coefficient + contribution_from_root;

            quotient_coefficients_reversed.push(last_coefficient_found);
        }

        quotient_coefficients_reversed.reverse();

        // We check that the constant term is correct: -1 * root * constant term of q = constant term of p
        if -i8::try_from(*root).unwrap() * quotient_coefficients_reversed[0] != self.coefficients[0]
        {
            return Err(anyhow::anyhow!(
                "Fail to divide the polynomial by a root, constant terms do not add up"
            ));
        }

        Ok(Polynomial::from(quotient_coefficients_reversed.as_slice()))
    }

    /// Generate the G1Point representing the commit to the polynomial using setup artifacts.
    ///
    /// * `setup_artifacts` - List of setup artifacts for both elliptic curve groups. There must at least `degree + 1` artifacts.
    pub fn commit(&self, setup_artifacts: &[SetupArtifact]) -> Result<G1Point, anyhow::Error> {
        if self.degree() + 1 > setup_artifacts.len() {
            return Err(anyhow::anyhow!(
                "Setup does not allow for commitment generation of the polynomial. The polynomial degree is too high."
            ));
        }

        let mut commitment = blst::blst_p1::default();
        for (i, coefficient) in self.coefficients.iter().enumerate() {
            let coefficient_as_scalar = blst_scalar_from_i8_as_abs(*coefficient);
            let setup_point = &setup_artifacts[i].g1;

            let mut contribution = blst::blst_p1::default();
            unsafe {
                blst::blst_p1_mult(
                    &mut contribution,
                    setup_point.as_raw_ptr(),
                    coefficient_as_scalar.b.as_ptr(),
                    coefficient_as_scalar.b.len() * 8,
                );
            };
            if *coefficient < 0 {
                unsafe {
                    blst::blst_p1_cneg(&mut contribution, true);
                }
            }
            unsafe {
                blst::blst_p1_add_or_double(&mut commitment, &commitment, &contribution);
            };
        }

        Ok(commitment.into())
    }
}

pub fn blst_scalar_from_i8_as_abs(a: i8) -> blst::blst_scalar {
    let mut le_bytes = [0; 48];
    le_bytes[0] = a.unsigned_abs();
    let mut scalar: blst::blst_scalar = blst::blst_scalar::default();
    unsafe { blst::blst_scalar_from_le_bytes(&mut scalar, le_bytes.as_ptr(), le_bytes.len()) };
    scalar
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

fn display_non_zero_coefficient(c: i8, degree: usize) -> String {
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
