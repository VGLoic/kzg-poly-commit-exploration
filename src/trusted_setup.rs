use num_bigint::BigUint;
use serde::{
    self, Deserialize, Serialize,
    de::{self, MapAccess, SeqAccess, Visitor},
    ser::SerializeStruct,
};

#[derive(Debug)]
pub struct SetupArtifactsGenerator {
    secret: BigUint,
    is_at_power_zero: bool,
    current_s_powered: BigUint,
}

impl SetupArtifactsGenerator {
    /// Creates a new generator for trusted setup artifacts
    ///
    /// * `secret` - Secret in order to generate artifacts, in big endian bytes
    pub fn new(secret: [u8; 48]) -> Self {
        Self {
            secret: BigUint::from_bytes_be(&secret),
            is_at_power_zero: true,
            current_s_powered: BigUint::from(1u8),
        }
    }
}

#[derive(Debug)]
pub struct SetupArtifact {
    pub g1: blst::blst_p1,
    pub g2: blst::blst_p2,
}

impl Serialize for SetupArtifact {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("SetupArtifact", 2)?;

        let mut compressed_p1 = [0; 48];
        unsafe {
            blst::blst_p1_compress(compressed_p1.as_mut_ptr(), &self.g1);
        };
        state.serialize_field("g1", &compressed_p1[..])?;

        let mut compressed_p2 = [0; 96];
        unsafe {
            blst::blst_p2_compress(compressed_p2.as_mut_ptr(), &self.g2);
        };
        state.serialize_field("g2", &compressed_p2[..])?;

        state.end()
    }
}

impl<'de> Deserialize<'de> for SetupArtifact {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            G1,
            G2,
        }

        struct SetupArtifactVisitor;

        fn vec_to_blst_p1(v: Vec<u8>) -> Result<blst::blst_p1, anyhow::Error> {
            if v.len() != 48 {
                return Err(anyhow::anyhow!(
                    "Invalid length, expected 48, got {}",
                    v.len()
                ));
            }

            let mut compressed_p1 = [0u8; 48];
            compressed_p1.copy_from_slice(&v);
            let mut uncompressed_p1_affine = blst::blst_p1_affine::default();
            unsafe {
                match blst::blst_p1_uncompress(&mut uncompressed_p1_affine, compressed_p1.as_ptr())
                {
                    blst::BLST_ERROR::BLST_SUCCESS => Ok(()),
                    other => Err(other),
                }
            }
            .map_err(|err| anyhow::anyhow!("Got error while uncompressing: {err:?}"))?;

            let mut uncompressed_p1 = blst::blst_p1::default();
            unsafe {
                blst::blst_p1_from_affine(&mut uncompressed_p1, &uncompressed_p1_affine);
            };
            Ok(uncompressed_p1)
        }

        fn vec_to_blst_p2(v: Vec<u8>) -> Result<blst::blst_p2, anyhow::Error> {
            if v.len() != 96 {
                return Err(anyhow::anyhow!(
                    "Invalid length, expected 96, got {}",
                    v.len()
                ));
            }

            let mut compressed_p2 = [0u8; 96];
            compressed_p2.copy_from_slice(&v);
            let mut uncompressed_p2_affine = blst::blst_p2_affine::default();
            unsafe {
                match blst::blst_p2_uncompress(&mut uncompressed_p2_affine, compressed_p2.as_ptr())
                {
                    blst::BLST_ERROR::BLST_SUCCESS => Ok(()),
                    other => Err(other),
                }
            }
            .map_err(|err| anyhow::anyhow!("Got error while uncompressing: {err:?}"))?;

            let mut uncompressed_p2 = blst::blst_p2::default();
            unsafe {
                blst::blst_p2_from_affine(&mut uncompressed_p2, &uncompressed_p2_affine);
            };
            Ok(uncompressed_p2)
        }

        impl<'de> Visitor<'de> for SetupArtifactVisitor {
            type Value = SetupArtifact;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct SetupArtifact")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<SetupArtifact, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let raw_g1: Vec<u8> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let g1 = vec_to_blst_p1(raw_g1).map_err(de::Error::custom)?;

                let raw_g2: Vec<u8> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let g2 = vec_to_blst_p2(raw_g2).map_err(de::Error::custom)?;

                Ok(SetupArtifact { g1, g2 })
            }

            fn visit_map<V>(self, mut map: V) -> Result<SetupArtifact, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut raw_g1: Option<Vec<u8>> = None;
                let mut raw_g2: Option<Vec<u8>> = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::G1 => {
                            if raw_g1.is_some() {
                                return Err(de::Error::duplicate_field("g1"));
                            }
                            raw_g1 = Some(map.next_value()?);
                        }
                        Field::G2 => {
                            if raw_g2.is_some() {
                                return Err(de::Error::duplicate_field("g2"));
                            }
                            raw_g2 = Some(map.next_value()?);
                        }
                    }
                }

                let raw_g1 = raw_g1.ok_or_else(|| de::Error::missing_field("g1"))?;
                let g1 = vec_to_blst_p1(raw_g1).map_err(de::Error::custom)?;

                let raw_g2 = raw_g2.ok_or_else(|| de::Error::missing_field("g2"))?;
                let g2 = vec_to_blst_p2(raw_g2).map_err(de::Error::custom)?;

                Ok(SetupArtifact { g1, g2 })
            }
        }

        const FIELDS: &[&str] = &["g1", "g2"];
        deserializer.deserialize_struct("SetupArtifact", FIELDS, SetupArtifactVisitor)
    }
}

impl Iterator for SetupArtifactsGenerator {
    type Item = SetupArtifact;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_at_power_zero {
            self.is_at_power_zero = false;

            return Some(SetupArtifact {
                g1: unsafe { *blst::blst_p1_generator() },
                g2: unsafe { *blst::blst_p2_generator() },
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
            g1: g1_artifact,
            g2: g2_artifact,
        })
    }
}
