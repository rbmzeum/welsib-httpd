use crate::signature::Signature;
use crate::point::Point;
// use crate::registry::{AVAILABLE_STATIC_FILE_URI, VERIFY_KEY, WEB_RESOURCE_DIR}; // TODO: load from config
use crate::config::Config;
use crate::helpers::compress;
use std::collections::HashMap;
use crate::welsib::*;

pub struct WelsibResource {
    pub list: HashMap<String, Vec<u8>>,
    pub gzipped_list: HashMap<String, Vec<u8>>,
    pub sign_list: HashMap<String, Signature>,
    pub verify_key: Point,
    pub available_static_file_uri: Vec<String>,
}

impl WelsibResource {
    pub fn load(has_check_signatures: bool, config: &Config) -> std::io::Result<Self> {
        let mut list = HashMap::new();
        let mut gzipped_list = HashMap::new();
        let mut sign_list = HashMap::new();

        let verify_key = Point {
            x: num_bigint::BigInt::parse_bytes(config.verify_key.0.as_bytes(), 16).unwrap(),
            y: num_bigint::BigInt::parse_bytes(config.verify_key.1.as_bytes(), 16).unwrap(),
        };

        let web_dir = std::env::current_dir()?.join(&config.web_resource_dir);
        if web_dir.exists() {
            for uri in [
                &config.available_static_file_uri[..],
                &["/404.html".to_string(), "/400.html".to_string(), "/500.html".to_string()],
            ]
            .concat()
            {
                let filename = if uri.eq(&String::from("/")) {
                    "home.html"
                } else {
                    &uri[1..]
                };
                let file = web_dir.clone().join(filename);
                let bytes = std::fs::read(&file)?;

                if has_check_signatures {
                    let sig_file = web_dir.clone().join(format!("{}.sig", filename));
                    let signature_bytes = std::fs::read(&sig_file)?;
                    let hash = unsafe { digest(&bytes) };
                    if signature_bytes.len() < 128 {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "Не верный объём данных цифровой подписи, необходимо 128 байт, получено {} из файла {}.",
                                &signature_bytes.len(),
                                sig_file.to_str().unwrap()
                            ),
                        ));
                    }
                    let sign = Signature::from_be_bytes(&signature_bytes);
                    let is_valid =
                        unsafe { verify(&hash, &sign.to_be_bytes(), &verify_key.to_be_bytes()) };
                    if !is_valid {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            format!(
                                "Цифровая подпись {} не верна.",
                                sig_file.to_str().unwrap()
                            ),
                        ));
                    }
                    sign_list.insert(uri.clone(), sign);
                }

                let compressed_bytes = compress(&bytes)?;

                list.insert(uri.clone(), bytes);
                gzipped_list.insert(uri.clone(), compressed_bytes);
            }
        }

        Ok(Self {
            list,
            gzipped_list,
            sign_list,
            verify_key: config.get_verify_key(),
            available_static_file_uri: config.available_static_file_uri.clone(),
        })
    }
}