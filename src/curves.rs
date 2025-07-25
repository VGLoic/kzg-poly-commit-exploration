use std::ops::Deref;

use serde::{
    Deserialize, Serialize,
    de::{self, Visitor},
};

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
    pub fn as_raw_ptr(&self) -> *const blst::blst_p1 {
        &self.0
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
    pub fn as_raw_ptr(&self) -> *const blst::blst_p2 {
        &self.0
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
