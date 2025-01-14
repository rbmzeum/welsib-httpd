pub fn vec2hex(data: Vec<u8>) -> String {
    data.iter().map(|b| format!("{:02x}", b)).collect::<String>()
}
