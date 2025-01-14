use num_bigint::BigInt;

pub fn bigint2vec(x: BigInt) -> Vec<u8> {
    x.iter_u64_digits()
        .rev()
        .map(|v| v.to_be_bytes().to_vec())
        .collect::<Vec<Vec<u8>>>()
        .concat()
}
