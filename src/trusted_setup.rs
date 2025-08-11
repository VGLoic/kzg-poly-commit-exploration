use serde::{self, Deserialize, Serialize};

use super::{
    curves::{G1Point, G2Point},
    scalar::Scalar,
    curves
};

#[derive(Debug)]
pub struct SetupArtifactsGenerator {
    secret: Scalar,
    is_at_power_zero: bool,
    current_s_powered: Scalar,
}

impl SetupArtifactsGenerator {
    /// Creates a new generator for trusted setup artifacts
    ///
    /// * `secret` - Secret used to generate artifacts, in big endian bytes
    pub fn new(secret: [u8; 32]) -> Self {
        let mut one_le_bytes = [0; 32];
        one_le_bytes[0] = 1;
        Self {
            secret: Scalar::from_be_bytes(secret),
            is_at_power_zero: true,
            current_s_powered: Scalar::from_le_bytes(one_le_bytes),
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
                g1: G1Point::from_i128(1),
                g2: G2Point::from_i128(1),
            });
        }

        self.current_s_powered = self.current_s_powered.mul(&self.secret);

        let s_powered_le_bytes = self.current_s_powered.to_le_bytes();

        let mut g1_artifact = blst::blst_p1::default();
        unsafe {
            blst::blst_p1_mult(
                &mut g1_artifact,
                blst::blst_p1_generator(),
                s_powered_le_bytes.as_ptr(),
                s_powered_le_bytes.len() * 8,
            );
        };

        let mut g2_artifact = blst::blst_p2::default();
        unsafe {
            blst::blst_p2_mult(
                &mut g2_artifact,
                blst::blst_p2_generator(),
                s_powered_le_bytes.as_ptr(),
                s_powered_le_bytes.len() * 8,
            );
        };

        Some(SetupArtifact {
            g1: g1_artifact.into(),
            g2: g2_artifact.into(),
        })
    }
}
