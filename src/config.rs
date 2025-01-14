use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::fs::File;
use std::io::{Read, Write};
use crate::point::Point;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub available_static_file_uri: Vec<String>,
    pub verify_key: (String, String),
    pub web_resource_dir: PathBuf,
    pub path_cert: PathBuf,
    pub path_key: PathBuf,
    pub path_ca: PathBuf,
}

impl Config {
    pub fn load(config_path: &PathBuf) -> std::io::Result<Self> {
        let mut file = File::open(config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config: Config = serde_json::from_str(&contents)?;
        Ok(config)
    }

    pub fn save(&self, config_path: &PathBuf) -> std::io::Result<()> {
        // Создаём все директории, если они не существуют
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Сериализуем конфигурацию в JSON
        let json = serde_json::to_string_pretty(self)?;

        // Создаём файл и записываем в него JSON
        let mut file = File::create(config_path)?;
        file.write_all(json.as_bytes())?;

        Ok(())
    }

    pub fn get_verify_key(&self) -> Point {
        Point {
            x: num_bigint::BigInt::parse_bytes(self.verify_key.0.as_bytes(), 16).unwrap(),
            y: num_bigint::BigInt::parse_bytes(self.verify_key.1.as_bytes(), 16).unwrap(),
        }
    }
}