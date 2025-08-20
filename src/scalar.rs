use serde::{
    Deserialize, Serialize,
    de::{self, Visitor},
    ser::Error,
};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct Scalar(blst::blst_fr);

const R_AS_HEX: &str = "73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001";

fn le_bytes_to_hex(a: &[u8]) -> String {
    let mut out = "".to_string();
    for b in a.iter().rev() {
        out += format!("{b:02x?}").as_str();
    }
    out
}
fn be_bytes_to_hex(a: &[u8]) -> String {
    let mut out = "".to_string();
    for b in a.iter() {
        out += format!("{b:02x?}").as_str();
    }
    out
}

impl From<i128> for Scalar {
    fn from(value: i128) -> Self {
        let mut unsigned_le_bytes = [0u8; 32];
        unsigned_le_bytes[..16].copy_from_slice(&value.unsigned_abs().to_le_bytes());
        let unsigned_hexa = le_bytes_to_hex(&unsigned_le_bytes);
        let mut fr = blst::blst_fr::default();
        unsafe {
            blst::blst_fr_from_hexascii(&mut fr, unsigned_hexa.as_ptr());
        }
        if value > 0 {
            return Self(fr);
        }

        let mut r = blst::blst_fr::default();
        unsafe {
            blst::blst_fr_from_hexascii(&mut r, R_AS_HEX.as_ptr());
            blst::blst_fr_sub(&mut fr, &r, &fr);
        };

        Self(fr)
    }
}

impl Scalar {
    /// Creates a scalar from low endian bytes
    ///
    /// * `b` - Low endian byte array of length 32
    pub fn from_le_bytes(b: [u8; 32]) -> Self {
        let hexa = le_bytes_to_hex(&b);
        let mut fr = blst::blst_fr::default();
        unsafe {
            blst::blst_fr_from_hexascii(&mut fr, hexa.as_ptr());
        }
        Self(fr)
    }

    /// Creates a scalar from big endian bytes
    ///
    /// * `b` - Big endian byte array of length 32
    pub fn from_be_bytes(b: [u8; 32]) -> Self {
        let mut fr = blst::blst_fr::default();
        let hexa = be_bytes_to_hex(&b);
        unsafe {
            blst::blst_fr_from_hexascii(&mut fr, hexa.as_ptr());
        }
        Self(fr)
    }

    /// Creates a scalar from a i128
    ///
    /// * `a` - i128 value
    pub fn from_i128(a: i128) -> Self {
        Self::from(a)
    }

    /// Returns the low endian bytes representation of the scalar
    pub fn to_le_bytes(&self) -> [u8; 32] {
        let mut scalar = blst::blst_scalar::default();
        unsafe {
            blst::blst_scalar_from_fr(&mut scalar, &self.0);
        }
        let mut le_bytes = [0u8; 32];
        unsafe {
            blst::blst_lendian_from_scalar(le_bytes.as_mut_ptr(), &scalar);
        }
        le_bytes
    }

    /// Returns the big endian bytes representation of the scalar
    pub fn to_be_bytes(&self) -> [u8; 32] {
        let mut scalar = blst::blst_scalar::default();
        unsafe {
            blst::blst_scalar_from_fr(&mut scalar, &self.0);
        }
        let mut be_bytes = [0u8; 32];
        unsafe {
            blst::blst_bendian_from_scalar(be_bytes.as_mut_ptr(), &scalar);
        }
        be_bytes
    }

    /// Returns a new scalar obtained by the multiplication of self and another scalar
    ///
    /// - `other` - Other scalar to perform the operation
    pub fn mul(&self, other: &Self) -> Self {
        let mut out = blst::blst_fr::default();
        unsafe {
            blst::blst_fr_mul(&mut out, &self.0, &other.0);
        };
        Self(out)
    }

    /// Returns a new scalar obtained by the multiplication of self by itself a given number of times
    ///
    /// - `n` - Number of times to multiply self by itself
    pub fn pow(&self, n: usize) -> Self {
        let mut out = Scalar::from_i128(1).0;
        for _ in 0..n {
            unsafe {
                blst::blst_fr_mul(&mut out, &out, &self.0);
            }
        }
        Scalar(out)
    }

    /// Returns a new scalar obtained by the addition of self and another scalar
    ///
    /// - `other` - Other scalar to perform the operation
    pub fn add(&self, other: &Self) -> Self {
        let mut out = blst::blst_fr::default();
        unsafe {
            blst::blst_fr_add(&mut out, &self.0, &other.0);
        }
        Scalar(out)
    }

    /// Returns a new scalar obtained by the subtraction of self by another scalar
    ///
    /// - `other` - Other scalar to perform the subtraction
    pub fn sub(&self, other: &Self) -> Self {
        let mut out = blst::blst_fr::default();
        unsafe {
            blst::blst_fr_sub(&mut out, &self.0, &other.0);
        }
        Scalar(out)
    }

    /// Returns a new scalar obtained by the negation of self
    pub fn neg(&self) -> Self {
        let mut out = blst::blst_fr::default();
        unsafe {
            blst::blst_fr_cneg(&mut out, &self.0, true);
        }
        Scalar(out)
    }

    /// Returns true if self is the representation of zero, false otherwise
    pub fn is_zero(&self) -> bool {
        self.0 == blst::blst_fr::default()
    }
}

impl Serialize for Scalar {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.to_le_bytes())
    }
}

impl<'de> Deserialize<'de> for Scalar {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ScalarVisitor;

        impl<'de> Visitor<'de> for ScalarVisitor {
            type Value = Scalar;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Sequence of u8")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut elements: Vec<u8> = vec![];

                while let Some(a) = seq.next_element()? {
                    elements.push(a)
                }

                if elements.len() != 32 {
                    return Err(de::Error::custom(format!(
                        "Invalid byte array, expected length 32, got {}",
                        elements.len()
                    )));
                }

                let mut le_bytes = [0u8; 32];
                le_bytes.copy_from_slice(&elements[0..32]);

                Ok(Scalar::from_le_bytes(le_bytes))
            }
        }

        deserializer.deserialize_seq(ScalarVisitor)
    }
}

impl Display for Scalar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let displayed = display_le_bytes(&self.to_le_bytes())
            .map_err(|e| std::fmt::Error::custom(e.to_string()))?;

        write!(f, "{displayed}")
    }
}
fn display_le_bytes(le_bytes: &[u8]) -> Result<String, anyhow::Error> {
    let mut digits: Vec<u8> = vec![];

    let mut quotient = le_bytes.to_vec();

    while quotient.iter().any(|&byte| byte != 0) {
        let mut next_digit: u16 = 0;
        for byte in quotient.iter_mut().rev() {
            let to_be_divided = (next_digit << 8) | *byte as u16;
            *byte = (to_be_divided / 10) as u8;
            next_digit = to_be_divided % 10;
        }
        digits.push((next_digit as u8) + b'0')
    }

    digits.reverse();

    String::from_utf8(digits).map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use fake::{Fake, Faker};
    use num_bigint::BigUint;

    use super::*;

    #[test]
    fn test_i128_to_scalar_using_le() {
        let a: i128 = Faker.fake();
        let scalar = Scalar::from_i128(a);
        let recovered_le_bytes = scalar.to_le_bytes();
        let mut expected_le_bytes = [0u8; 32];
        if a > 0 {
            expected_le_bytes[..16].copy_from_slice(&a.to_le_bytes());
        } else {
            let unsigned_a_le_bytes = a.unsigned_abs().to_le_bytes();
            let r_be_bytes = hex::decode(R_AS_HEX).unwrap();
            // Always safe to do as R is bigger than i128 maximum value
            let expected_big_uint =
                BigUint::from_bytes_be(&r_be_bytes) - BigUint::from_bytes_le(&unsigned_a_le_bytes);
            // It will always fit in 32 bytes as R fits in 32 bytes and is in any case larger than the unsigned part of the input
            expected_le_bytes[..].copy_from_slice(expected_big_uint.to_bytes_le().as_slice());
        }
        assert_eq!(recovered_le_bytes, expected_le_bytes);
    }

    #[test]
    fn test_u128_to_scalar_using_le() {
        let a: u128 = Faker.fake();
        let mut le_bytes = [0u8; 32];
        le_bytes[..16].copy_from_slice(&a.to_le_bytes());
        let scalar = Scalar::from_le_bytes(le_bytes);
        let recovered_le_bytes = scalar.to_le_bytes();
        assert_eq!(recovered_le_bytes, le_bytes);
    }

    #[test]
    fn test_u128_to_scalar_using_be() {
        let a: u128 = Faker.fake();
        let mut be_bytes = [0u8; 32];
        be_bytes[..16].copy_from_slice(&a.to_le_bytes());
        be_bytes.reverse();
        let scalar = Scalar::from_be_bytes(be_bytes);
        let recovered_be_bytes = scalar.to_be_bytes();
        assert_eq!(recovered_be_bytes, be_bytes);
    }

    #[test]
    fn test_display_scalar() {
        let r_be_bytes = hex::decode(R_AS_HEX).unwrap();
        let mut a: [u8; 32] = Faker.fake();
        if a[31] >= r_be_bytes[0] {
            a[31] = r_be_bytes[0] - 1
        }
        let from_big_uint = format!("{}", BigUint::from_bytes_le(&a));
        let from_scalar = format!("{}", Scalar::from_le_bytes(a));
        assert_eq!(from_big_uint, from_scalar);
    }
}
