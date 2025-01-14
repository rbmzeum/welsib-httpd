use num_bigint::BigInt;

#[derive(Clone, Debug, PartialEq)]
pub struct Point {
    pub x: BigInt,
    pub y: BigInt,
}

impl Point {
    pub fn from_be_bytes(bytes: &Vec<u8>) -> Self {
        Self {
            x: BigInt::from_bytes_be(num_bigint::Sign::Plus, &bytes[0..64]),
            y: BigInt::from_bytes_be(num_bigint::Sign::Plus, &bytes[64..128]),
        }
    }

    pub fn to_be_bytes(&self) -> Vec<u8> {
        [
            self.x.clone().iter_u64_digits().rev().map(|v| { v.to_be_bytes().to_vec() }).collect::<Vec<Vec<u8>>>().concat(),
            self.y.clone().iter_u64_digits().rev().map(|v| { v.to_be_bytes().to_vec() }).collect::<Vec<Vec<u8>>>().concat(),
        ].concat()
    }

    pub fn to_hex(&self) -> String {
        let x = self.x.to_str_radix(16);
        let y = self.y.to_str_radix(16);
        format!("X={},Y={}", x, y)
    }
}