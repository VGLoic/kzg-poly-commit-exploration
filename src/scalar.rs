#[derive(Debug)]
pub struct Scalar {
    as_fr: blst::blst_fr,
}

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

impl Scalar {
    pub fn from_le_bytes(b: [u8; 32]) -> Self {
        let hexa = le_bytes_to_hex(&b);
        let mut fr = blst::blst_fr::default();
        unsafe {
            blst::blst_fr_from_hexascii(&mut fr, hexa.as_ptr());
        }
        Self { as_fr: fr }
    }

    pub fn from_be_bytes(b: [u8; 32]) -> Self {
        let mut fr = blst::blst_fr::default();
        let hexa = be_bytes_to_hex(&b);
        unsafe {
            blst::blst_fr_from_hexascii(&mut fr, hexa.as_ptr());
        }
        Self { as_fr: fr }
    }

    pub fn from_i128(a: i128) -> Self {
        let mut unsigned_le_bytes = [0u8; 32];
        unsigned_le_bytes[..16].copy_from_slice(&a.unsigned_abs().to_le_bytes());
        let unsigned_hexa = le_bytes_to_hex(&unsigned_le_bytes);
        let mut fr = blst::blst_fr::default();
        unsafe {
            blst::blst_fr_from_hexascii(&mut fr, unsigned_hexa.as_ptr());
        }
        if a > 0 {
            return Self { as_fr: fr };
        }

        let mut r = blst::blst_fr::default();
        unsafe {
            blst::blst_fr_from_hexascii(&mut r, R_AS_HEX.as_ptr());
            blst::blst_fr_sub(&mut fr, &r, &fr);
        };

        Self { as_fr: fr }
    }

    pub fn to_le_bytes(&self) -> [u8; 32] {
        let mut scalar = blst::blst_scalar::default();
        unsafe {
            blst::blst_scalar_from_fr(&mut scalar, &self.as_fr);
        }
        let mut le_bytes = [0u8; 32];
        unsafe {
            blst::blst_lendian_from_scalar(le_bytes.as_mut_ptr(), &scalar);
        }
        le_bytes
    }

    pub fn to_be_bytes(&self) -> [u8; 32] {
        let mut scalar = blst::blst_scalar::default();
        unsafe {
            blst::blst_scalar_from_fr(&mut scalar, &self.as_fr);
        }
        let mut be_bytes = [0u8; 32];
        unsafe {
            blst::blst_bendian_from_scalar(be_bytes.as_mut_ptr(), &scalar);
        }
        be_bytes
    }

    pub fn mul(&self, a: &Self) -> Self {
        let mut out = blst::blst_fr::default();
        unsafe {
            blst::blst_fr_mul(&mut out, &self.as_fr, &a.as_fr);
        };
        Self { as_fr: out }
    }
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
}
