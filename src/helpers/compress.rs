use flate2::write::GzEncoder;
use std::io::Write;

pub fn compress(bytes: &Vec<u8>) -> std::io::Result<Vec<u8>> {
    let buffer: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut compressor: GzEncoder<Vec<u8>> = GzEncoder::new(buffer, flate2::Compression::default());
    compressor.write_all(bytes.as_slice())?;
    Ok(compressor.finish()?)
}
