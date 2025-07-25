use serde::{Deserialize, Serialize};

use crate::curves::G1Point;

use super::trusted_setup::SetupArtifact;

#[derive(Debug, Deserialize, Serialize)]
pub struct Polynomial {
    coefficients: Vec<i8>,
}

impl From<&[i8]> for Polynomial {
    fn from(value: &[i8]) -> Self {
        let mut coefficients = vec![0; value.len()];
        coefficients.clone_from_slice(value);

        while let Some(last_value) = coefficients.last()
            && *last_value == 0
        {
            coefficients.pop();
        }

        Polynomial { coefficients }
    }
}

impl Polynomial {
    pub fn order(&self) -> usize {
        self.coefficients.len()
    }

    pub fn commit(&self, setup_artifacts: &[SetupArtifact]) -> Result<G1Point, anyhow::Error> {
        if self.order() > setup_artifacts.len() {
            return Err(anyhow::anyhow!(
                "Setup does not allow for commitment generation of the polynomial. The polynomial order is too high."
            ));
        }

        let mut commitment = blst::blst_p1::default();
        for (i, coefficient) in self.coefficients.iter().enumerate() {
            let coefficient_as_scalar = blst_scalar_from_u8(coefficient.unsigned_abs());
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

pub fn blst_scalar_from_u8(a: u8) -> blst::blst_scalar {
    let mut le_bytes = [0; 48];
    le_bytes[0] = a;
    let mut scalar = blst::blst_scalar::default();
    unsafe { blst::blst_scalar_from_le_bytes(&mut scalar, le_bytes.as_ptr(), le_bytes.len()) };
    scalar
}

impl std::fmt::Display for Polynomial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.coefficients.is_empty() {
            return write!(f, "0");
        }

        let higher_order_coefficient = self.coefficients[self.coefficients.len() - 1];
        let mut displayed = format!(
            "{}{}",
            if higher_order_coefficient < 0 {
                "-"
            } else {
                ""
            },
            display_non_zero_coefficient(higher_order_coefficient, self.coefficients.len() - 1)
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

fn display_non_zero_coefficient(c: i8, order: usize) -> String {
    let order_string = match order {
        0 => "".to_owned(),
        1 => "x".to_owned(),
        other => format!("x^{other}"),
    };
    if order > 0 && (c == 1 || c == -1) {
        return order_string;
    }
    format!("{}{order_string}", c.abs())
}
