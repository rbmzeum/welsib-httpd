use std::net::IpAddr;
use std::str::FromStr;
use std::path::PathBuf;

pub struct WelsibArguments {
    need_help: bool,
    has_ssl: bool,
    has_sig: bool,
    host: String,
    port: String,
    config_path: Option<PathBuf>, // Новое поле для хранения пути к конфигурационному файлу
}

impl WelsibArguments {
    pub fn init() -> Self {
        let mut need_help = false;
        let mut has_ssl = true;
        let mut has_sig = true;
        let mut host = String::from("127.0.0.1");
        let mut port = None;
        let mut config_path = None; // Инициализация переменной для хранения пути к конфигурационному файлу

        for argument in std::env::args() {
            // Need help
            if argument.eq("--help") || argument.eq("-h") {
                need_help = true;
            }

            // No SSL
            if argument.eq("--nossl") {
                has_ssl = false;
            }

            // No check signatures
            if argument.eq("--nosig") {
                has_sig = false;
            }

            // Host
            if argument.starts_with("--host=") {
                let h = argument.get(7..);
                match h {
                    Some(h) => {
                        let addr = IpAddr::from_str(h);
                        match addr {
                            Ok(addr) => {
                                if addr.is_ipv4() {
                                    host = String::from(h.trim());
                                }
                            }
                            _ => {}
                        }
                    }
                    None => {}
                }
            }

            // Port
            if argument.starts_with("--port=") {
                let p = argument.get(7..);
                match p {
                    Some(p) => {
                        let p = u16::from_str(p);
                        match p {
                            Ok(p) => {
                                if p > 1023 {
                                    port = Some(p);
                                } else {
                                    // TODO: log warning
                                }
                            }
                            _ => {}
                        }
                    }
                    None => {}
                }
            }

            // Config file
            if argument.starts_with("--config=") {
                let path = argument.get(9..); // Получаем путь, начиная с 9-го символа
                match path {
                    Some(path) => {
                        config_path = Some(PathBuf::from(path)); // Сохраняем путь в переменную
                    }
                    None => {}
                }
            }
        }

        Self {
            need_help,
            has_ssl,
            has_sig,
            host,
            port: match port {
                Some(port) => port.to_string(),
                None => (if has_ssl { 443 } else { 80 }).to_string(),
            },
            config_path, // Сохраняем путь к конфигурационному файлу
        }
    }

    pub fn has_ssl(&self) -> bool {
        self.has_ssl
    }

    pub fn has_check_signatures(&self) -> bool {
        self.has_sig
    }

    pub fn get_addr(&self) -> String {
        self.host.clone() + ":" + &self.port.clone()
    }

    pub fn need_help(&self) -> bool {
        self.need_help
    }

    // Новый метод для получения пути к конфигурационному файлу
    pub fn get_config_path(&self) -> Option<&PathBuf> {
        self.config_path.as_ref()
    }
}