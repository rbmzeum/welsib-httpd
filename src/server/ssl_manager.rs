use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use std::path::PathBuf;
use std::time::SystemTime;
use crate::net::WelsibStream;
use crate::config::Config;

/// Менеджер SSL, отвечающий за создание и управление SSL-акцептором.
pub struct SSLManager {
    acceptor: Option<SslAcceptor>,
    acceptor_modified: Option<SystemTime>,
}

impl SSLManager {
    /// Создает новый экземпляр SSLManager.
    pub fn new() -> Self {
        SSLManager {
            acceptor: None,
            acceptor_modified: None,
        }
    }

    /// Создает WelsibStream для защищенного соединения (если используется SSL).
    pub fn create_welsib_stream(&mut self, stream: std::net::TcpStream, config: &Config) -> Option<WelsibStream> {
        let (new_acceptor, new_acceptor_modified) = make_acceptor(self.acceptor_modified, config);
        if new_acceptor.is_some() {
            self.acceptor = new_acceptor;
            self.acceptor_modified = new_acceptor_modified;
        }

        match &self.acceptor {
            Some(acceptor) => match acceptor.accept(stream) {
                Ok(ssl_stream) => Some(WelsibStream {
                    ssl_stream: Some(ssl_stream),
                    tcp_stream: None,
                }),
                Err(e) => {
                    eprintln!("SSL\tHandshakeError<TcpStream>: {:#?}", e);
                    None
                }
            },
            None => None,
        }
    }
}

/// Создает SSL-акцептор, если сертификат изменился.
fn make_acceptor(previous_acceptor_modified: Option<SystemTime>, config: &Config) -> (Option<SslAcceptor>, Option<SystemTime>) {
    let path_cert = config.path_cert.clone();
    let path_key = config.path_key.clone();
    let path_ca = config.path_ca.clone();
    // let path_cert = PathBuf::from("/root/.acme.sh/welsib.ru_ecc/welsib.ru.cer");
    // let path_key = PathBuf::from("/root/.acme.sh/welsib.ru_ecc/welsib.ru.key");
    // let path_ca = PathBuf::from("/root/.acme.sh/welsib.ru_ecc/ca.cer");

    let new_modified = match std::fs::metadata(&path_cert) {
        Ok(metadata) => metadata.modified().ok(),
        _ => None,
    };

    if previous_acceptor_modified == new_modified {
        (None, None)
    } else {
        let mut acceptor_builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        acceptor_builder
            .set_private_key_file(path_key, SslFiletype::PEM)
            .unwrap();
        acceptor_builder
            .set_certificate_chain_file(path_cert)
            .unwrap();
        acceptor_builder.set_ca_file(path_ca).unwrap();
        acceptor_builder.check_private_key().unwrap();

        (Some(acceptor_builder.build()), new_modified)
    }
}