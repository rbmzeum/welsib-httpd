#[link(name="verify")]
unsafe extern {
    pub fn digest(data: &[u8]) -> Vec<u8>;
    pub fn digest_init();
    pub fn digest_update(bytes: &[u8]);
    pub fn digest_finalize() -> Vec<u8>;
    pub fn verify(hash: &Vec<u8>, signature: &Vec<u8>, verifying_key: &Vec<u8>) -> bool;
    pub fn is_not_test_signature_proof(signature_bytes: &Vec<u8>) -> bool;
    pub fn activation_proof(request_init_json: &Vec<u8>, multi_signature_bytes: &Vec<u8>) -> bool;
}