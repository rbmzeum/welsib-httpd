use num_bigint::BigInt;

pub fn vec2bigint(data: Vec<u8>) -> BigInt {
    BigInt::from_bytes_be(num_bigint::Sign::Plus, data.as_slice())
}
