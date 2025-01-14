use num_bigint::BigInt;

#[derive(Clone, Debug, PartialEq)]
pub struct Signature {
    pub r: BigInt,
    pub s: BigInt,
}

impl Signature {
    pub fn from_be_bytes(bytes: &Vec<u8>) -> Self {
        Self {
            r: BigInt::from_bytes_be(num_bigint::Sign::Plus, &bytes[0..64]),
            s: BigInt::from_bytes_be(num_bigint::Sign::Plus, &bytes[64..128]),
        }
    }

    pub fn to_be_bytes(&self) -> Vec<u8> {
        [
            self.r.clone().iter_u64_digits().rev().map(|v| { v.to_be_bytes().to_vec() }).collect::<Vec<Vec<u8>>>().concat(),
            self.s.clone().iter_u64_digits().rev().map(|v| { v.to_be_bytes().to_vec() }).collect::<Vec<Vec<u8>>>().concat(),
        ].concat()
    }
}