use std::ops::Deref;

use serde::{
    Deserialize, Serialize,
    de::{self, Visitor},
};

use crate::scalar::Scalar;

#[derive(Debug)]
pub struct G1Point(blst::blst_p1);

impl Deref for G1Point {
    type Target = blst::blst_p1;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<blst::blst_p1> for G1Point {
    fn from(value: blst::blst_p1) -> Self {
        G1Point(value)
    }
}

impl G1Point {
    /// Returns the wrapped point as raw pointer
    fn as_raw_ptr(&self) -> *const blst::blst_p1 {
        &self.0
    }

    /// Project a scalar to the G1 curve using the generator
    ///
    /// * `a` - Scalar to project
    pub fn from_i128(a: i128) -> Self {
        let scalar = blst_scalar_from_i128_as_abs(a);
        let mut out = blst::blst_p1::default();
        unsafe {
            blst::blst_p1_mult(
                &mut out,
                blst::blst_p1_generator(),
                scalar.b.as_ptr(),
                scalar.b.len() * 8,
            );
        };
        if a < 0 {
            unsafe {
                blst::blst_p1_cneg(&mut out, true);
            }
        }
        out.into()
    }

    /// Project a scalar to the G1 curve using the generator
    ///
    /// * `a` - Scalar to project
    pub fn from_scalar(a: Scalar) -> Self {
        let mut out = blst::blst_p1::default();
        unsafe {
            blst::blst_p1_mult(
                &mut out,
                blst::blst_p1_generator(),
                a.to_le_bytes().as_ptr(),
                256,
            );
        };
        out.into()
    }

    /// Subtract two points and give the result as a new point
    ///
    /// * `b` - G1 point to subtract from self
    pub fn sub(&self, b: &Self) -> Self {
        let mut out = blst::blst_p1::default();
        let mut neg_b = b.0;
        unsafe {
            blst::blst_p1_cneg(&mut neg_b, true);
            blst::blst_p1_add_or_double(&mut out, self.as_raw_ptr(), &neg_b);
        };
        out.into()
    }

    /// Add two points and give the result as a new point
    ///
    /// * `b` - G1 point to add to self
    pub fn add(&self, b: &Self) -> Self {
        let mut out = blst::blst_p1::default();
        unsafe {
            blst::blst_p1_add_or_double(&mut out, self.as_raw_ptr(), b.as_raw_ptr());
        };
        out.into()
    }

    /// Multiply a point by a scalar and give the result as a new point
    ///
    /// * `a` - Scalar that will multiply self
    pub fn mult(&self, a: &Scalar) -> Self {
        let mut out = blst::blst_p1::default();
        unsafe {
            blst::blst_p1_mult(&mut out, self.as_raw_ptr(), a.to_le_bytes().as_ptr(), 256);
        };
        out.into()
    }
}

impl Serialize for G1Point {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut compressed_p1 = [0; 48];
        unsafe {
            blst::blst_p1_compress(compressed_p1.as_mut_ptr(), self.as_raw_ptr());
        };
        serializer.serialize_bytes(&compressed_p1)
    }
}

impl<'de> Deserialize<'de> for G1Point {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct G1PointVisitor;

        fn bytes_to_blst_p1(v: &[u8]) -> Result<G1Point, anyhow::Error> {
            if v.len() != 48 {
                return Err(anyhow::anyhow!(
                    "Invalid length, expected 48, got {}",
                    v.len()
                ));
            }

            let mut compressed_p1 = [0u8; 48];
            compressed_p1.copy_from_slice(v);
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
            Ok(uncompressed_p1.into())
        }

        impl<'de> Visitor<'de> for G1PointVisitor {
            type Value = G1Point;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Sequence of u8")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut elements: Vec<u8> = vec![];

                while let Some(a) = seq.next_element()? {
                    elements.push(a)
                }

                bytes_to_blst_p1(&elements).map_err(de::Error::custom)
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                bytes_to_blst_p1(v).map_err(de::Error::custom)
            }

            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                bytes_to_blst_p1(v).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_seq(G1PointVisitor)
    }
}

#[derive(Debug)]
pub struct G2Point(blst::blst_p2);

impl From<blst::blst_p2> for G2Point {
    fn from(value: blst::blst_p2) -> Self {
        G2Point(value)
    }
}

impl G2Point {
    /// Returns the wrapped point as raw pointer
    fn as_raw_ptr(&self) -> *const blst::blst_p2 {
        &self.0
    }

    /// Project a scalar to the G2 curve using the generator
    ///
    /// * `a` - Scalar to project
    pub fn from_i128(a: i128) -> Self {
        let scalar = blst_scalar_from_i128_as_abs(a);
        let mut out = blst::blst_p2::default();
        unsafe {
            blst::blst_p2_mult(
                &mut out,
                blst::blst_p2_generator(),
                scalar.b.as_ptr(),
                scalar.b.len() * 8,
            );
        };
        if a < 0 {
            unsafe {
                blst::blst_p2_cneg(&mut out, true);
            }
        }
        out.into()
    }

    /// Project a scalar to the G2 curve using the generator
    ///
    /// * `a` - Scalar to project
    pub fn from_scalar(a: Scalar) -> Self {
        let mut out = blst::blst_p2::default();
        unsafe {
            blst::blst_p2_mult(
                &mut out,
                blst::blst_p2_generator(),
                a.to_le_bytes().as_ptr(),
                256,
            );
        };
        out.into()
    }

    /// Subtract two points and give the result as a new point
    ///
    /// * `b` - G2 point to subtract from self
    pub fn sub(&self, b: &Self) -> Self {
        let mut out = blst::blst_p2::default();
        let mut neg_b = b.0;
        unsafe {
            blst::blst_p2_cneg(&mut neg_b, true);
            blst::blst_p2_add_or_double(&mut out, self.as_raw_ptr(), &neg_b);
        };
        out.into()
    }
}

impl Deref for G2Point {
    type Target = blst::blst_p2;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for G2Point {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut compressed_p2 = [0; 96];
        unsafe {
            blst::blst_p2_compress(compressed_p2.as_mut_ptr(), self.as_raw_ptr());
        };
        serializer.serialize_bytes(&compressed_p2)
    }
}

impl<'de> Deserialize<'de> for G2Point {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct G2PointVisitor;

        fn bytes_to_blst_p2(v: &[u8]) -> Result<G2Point, anyhow::Error> {
            if v.len() != 96 {
                return Err(anyhow::anyhow!(
                    "Invalid length, expected 96, got {}",
                    v.len()
                ));
            }

            let mut compressed_p2 = [0u8; 96];
            compressed_p2.copy_from_slice(v);
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
            Ok(uncompressed_p2.into())
        }

        impl<'de> Visitor<'de> for G2PointVisitor {
            type Value = G2Point;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Sequence of u8")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut elements: Vec<u8> = vec![];

                while let Some(a) = seq.next_element()? {
                    elements.push(a)
                }

                bytes_to_blst_p2(&elements).map_err(de::Error::custom)
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                bytes_to_blst_p2(v).map_err(de::Error::custom)
            }

            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                bytes_to_blst_p2(v).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_seq(G2PointVisitor)
    }
}

fn blst_scalar_from_i128_as_abs(a: i128) -> blst::blst_scalar {
    let mut padded_bytes = [0u8; 48];
    padded_bytes[..16].copy_from_slice(&a.unsigned_abs().to_le_bytes());
    let mut scalar: blst::blst_scalar = blst::blst_scalar::default();
    unsafe {
        blst::blst_scalar_from_le_bytes(&mut scalar, padded_bytes.as_ptr(), padded_bytes.len())
    };
    scalar
}

pub fn bilinear_map(p1: &G1Point, p2: &G2Point) -> blst::blst_fp12 {
    let mut p1_affine = blst::blst_p1_affine::default();
    unsafe {
        blst::blst_p1_to_affine(&mut p1_affine, p1.as_raw_ptr());
    };
    let mut p2_affine = blst::blst_p2_affine::default();
    unsafe {
        blst::blst_p2_to_affine(&mut p2_affine, p2.as_raw_ptr());
    };

    let mut res = blst::blst_fp12::default();
    unsafe {
        blst::blst_miller_loop(&mut res, &p2_affine, &p1_affine);
        blst::blst_final_exp(&mut res, &res);
    };
    res
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_point_addition_and_scalar_multiplication() {
        unsafe {
            let g1 = blst::blst_p1_generator();

            let mut p1_via_addition = blst::blst_p1::default();
            blst::blst_p1_add_or_double(&mut p1_via_addition, g1, g1);

            let mut p1_via_multiplication = blst::blst_p1::default();
            let scalar_as_bytes = 2_u8.to_be_bytes();
            blst::blst_p1_mult(
                &mut p1_via_multiplication,
                g1,
                scalar_as_bytes.as_ptr(),
                scalar_as_bytes.len() * 8,
            );

            assert!(blst::blst_p1_in_g1(g1), "g1 must be in the first group");
            assert_eq!(
                p1_via_multiplication, p1_via_addition,
                "results must be the same via multiplication and via addition"
            );
            assert_ne!(
                p1_via_multiplication, *g1,
                "result must be different than g1"
            );
            assert!(
                blst::blst_p1_in_g1(&p1_via_multiplication),
                "result must be in first group"
            );
        }
    }

    #[test]
    fn test_compression_and_serialization() {
        unsafe {
            let g1 = blst::blst_p1_generator();

            let mut p1 = blst::blst_p1::default();
            blst::blst_p1_add_or_double(&mut p1, g1, g1);

            let mut compressed_p1 = [0; 48];
            blst::blst_p1_compress(compressed_p1.as_mut_ptr(), &p1);
            let mut uncompressed_p1_affine = blst::blst_p1_affine::default();
            match blst::blst_p1_uncompress(&mut uncompressed_p1_affine, compressed_p1.as_ptr()) {
                blst::BLST_ERROR::BLST_SUCCESS => {}
                other => {
                    println!("Got error while uncompressing: {other:?}");
                    panic!("Fail to uncompress")
                }
            };
            let mut uncompressed_p1 = blst::blst_p1::default();
            blst::blst_p1_from_affine(&mut uncompressed_p1, &uncompressed_p1_affine);
            assert_eq!(
                uncompressed_p1, p1,
                "result after uncompression must be equal to p1"
            );

            let mut serialized_p1 = [0; 96];
            blst::blst_p1_serialize(serialized_p1.as_mut_ptr(), &p1);
            let mut deserialized_p1_affine = blst::blst_p1_affine::default();
            match blst::blst_p1_deserialize(&mut deserialized_p1_affine, serialized_p1.as_ptr()) {
                blst::BLST_ERROR::BLST_SUCCESS => {}
                other => {
                    println!("Got error while deserializing: {other:?}",);
                    panic!("Fail to deserialize")
                }
            };

            let mut deserialized_p1 = blst::blst_p1::default();
            blst::blst_p1_from_affine(&mut deserialized_p1, &deserialized_p1_affine);
            assert_eq!(
                deserialized_p1, p1,
                "result after deserialization must be equal to p1"
            );
        }
    }
}
