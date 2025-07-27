use num_bigint::BigUint;
use serde::{self, Deserialize, Serialize};

use super::curves;

#[derive(Debug)]
pub struct SetupArtifactsGenerator {
    secret: BigUint,
    is_at_power_zero: bool,
    current_s_powered: BigUint,
}

impl SetupArtifactsGenerator {
    /// Creates a new generator for trusted setup artifacts
    ///
    /// * `secret` - Secret used to generate artifacts, in big endian bytes
    pub fn new(secret: [u8; 48]) -> Self {
        Self {
            secret: BigUint::from_bytes_be(&secret),
            is_at_power_zero: true,
            current_s_powered: BigUint::from(1u8),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupArtifact {
    pub g1: curves::G1Point,
    pub g2: curves::G2Point,
}

impl Iterator for SetupArtifactsGenerator {
    type Item = SetupArtifact;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_at_power_zero {
            self.is_at_power_zero = false;

            return Some(SetupArtifact {
                g1: unsafe { *blst::blst_p1_generator() }.into(),
                g2: unsafe { *blst::blst_p2_generator() }.into(),
            });
        }

        self.current_s_powered *= &self.secret;

        let s_powered_be_bytes = self.current_s_powered.to_bytes_be();
        let mut s_powered_as_scalar = blst::blst_scalar::default();
        unsafe {
            blst::blst_scalar_from_be_bytes(
                &mut s_powered_as_scalar,
                s_powered_be_bytes.as_ptr(),
                s_powered_be_bytes.len(),
            );
        };
        let mut g1_artifact = blst::blst_p1::default();
        unsafe {
            blst::blst_p1_mult(
                &mut g1_artifact,
                blst::blst_p1_generator(),
                s_powered_as_scalar.b.as_ptr(),
                s_powered_as_scalar.b.len() * 8,
            );
        };

        let mut g2_artifact = blst::blst_p2::default();
        unsafe {
            blst::blst_p2_mult(
                &mut g2_artifact,
                blst::blst_p2_generator(),
                s_powered_as_scalar.b.as_ptr(),
                s_powered_as_scalar.b.len() * 8,
            );
        };

        Some(SetupArtifact {
            g1: g1_artifact.into(),
            g2: g2_artifact.into(),
        })
    }
}
