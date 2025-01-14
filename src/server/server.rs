use std::collections::VecDeque;
use std::net::TcpListener;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;

use crate::dispatcher::Dispatcher;
use crate::resource::WelsibResource;
use crate::context::WelsibContext;
use crate::net::WelsibStream;
use crate::helpers::WelsibArguments;
use crate::channel::WelsibChannel;
use crate::config::Config;

use super::ssl_manager::SSLManager;
use super::help::Help;

/// Основной класс, отвечающий за запуск сервера и управление его компонентами.
pub struct Server {
    arguments: WelsibArguments,
    dispatcher: Arc<Mutex<Dispatcher>>,
    resource: Arc<Mutex<WelsibResource>>,
    sender: Arc<Mutex<Sender<bool>>>,
    receiver: Arc<Mutex<Receiver<bool>>>,
    has_executor_connected: Arc<Mutex<bool>>,
    api_request_elapsed_time: Arc<Mutex<SystemTime>>,
    channels: Option<Arc<Mutex<VecDeque<WelsibChannel>>>>,
    ssl_manager: SSLManager,
    config: Config,
}

impl Server {
    /// Создает новый экземпляр сервера.
    pub fn new() -> std::io::Result<Self> {
        let arguments = WelsibArguments::init();
        if arguments.need_help() {
            Help::display();
            std::process::exit(0);
        }

        // Загрузка конфигурации
        let config = match arguments.get_config_path() {
            Some(config_path) if std::path::Path::new(config_path).exists() => {
                Config::load(config_path)?
            }
            _ => {
                let home_path = PathBuf::from("/srv/welsib-httpd/web/home.html");
                if let Some(home_dir) = home_path.parent() {
                    std::fs::create_dir_all(home_dir)?;
                    let file_404 = home_dir.join("404.html");
                    let file_400 = home_dir.join("400.html");
                    let file_500 = home_dir.join("500.html");
                    if !file_404.exists() {
                        let mut file = File::create(&file_404)?;
                        file.write_all(String::from("<h1>404</h1>").as_bytes())?;
                    }
                    if !file_400.exists() {
                        let mut file = File::create(&file_400)?;
                        file.write_all(String::from("<h1>400</h1>").as_bytes())?;
                    }
                    if !file_500.exists() {
                        let mut file = File::create(&file_500)?;
                        file.write_all(String::from("<h1>500</h1>").as_bytes())?;
                    }
                }
                // Создаём файл home.html и записываем в него текст
                let mut home_file = File::create(&home_path)?;
                home_file.write_all(String::from("<h1>Welsib Web Server/v0.1.0.0</h1>").as_bytes())?;

                let config = Config {
                    available_static_file_uri: vec![String::from("/")], // "/" => "/home.html"
                    verify_key: (
                        String::from("9e4c452444fb1de73afc6e3c057b6c3ae6f01c179a10248a283985d08636d7b0c9e28968fafc1323f35985267080631b64aa90363a745ef0549faa1ed87cf219"),
                        String::from("ca4dbd8e97e95550ca4452c7aca427796752433050c68fab4b3c9ce236a03ae79f050e775f37eeedaf9a57fc721aa823540a6a77340e533957e47cc0354d51fa")
                    ),
                    web_resource_dir: home_path.parent().unwrap().to_path_buf(),
                    path_cert: PathBuf::from("/root/.acme.sh/welsib.ru_ecc/welsib.ru.cer"),
                    path_key: PathBuf::from("/root/.acme.sh/welsib.ru_ecc/welsib.ru.key"),
                    path_ca: PathBuf::from("/root/.acme.sh/welsib.ru_ecc/ca.cer"),
                };
                config.save(&PathBuf::from("/etc/welsib-httpd/config.json"))?;
                config
            },
        };

        let dispatcher = Arc::new(Mutex::new(Dispatcher::new()));
        let resource = Arc::new(Mutex::new(WelsibResource::load(arguments.has_check_signatures(), &config)?));

        let (sender, receiver) = channel();
        let sender = Arc::new(Mutex::new(sender));
        let receiver = Arc::new(Mutex::new(receiver));

        let has_executor_connected = Arc::new(Mutex::new(false));
        let api_request_elapsed_time = Arc::new(Mutex::new(SystemTime::now()));
        let channels = Some(Arc::new(Mutex::new(VecDeque::new())));

        let ssl_manager = SSLManager::new();

        Ok(Server {
            arguments,
            dispatcher,
            resource,
            sender,
            receiver,
            has_executor_connected,
            api_request_elapsed_time,
            channels,
            ssl_manager,
            config,
        })
    }

    /// Запускает сервер и начинает обработку входящих соединений.
    pub fn run(&mut self) -> std::io::Result<()> {
        let addr = self.arguments.get_addr();
        let listener = TcpListener::bind(addr)?;

        self.init_dispatcher()?;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => self.handle_stream(stream)?,
                Err(e) => eprintln!("TcpStream\tincoming error: {:#?}", e),
            }
        }

        Ok(())
    }

    /// Инициализирует диспетчер и создает контекст для обработки запросов.
    fn init_dispatcher(&self) -> std::io::Result<()> {
        let context = WelsibContext::new(
            None,
            self.channels.clone(),
            self.resource.clone(),
            self.has_executor_connected.clone(),
            self.api_request_elapsed_time.clone(),
            self.sender.clone(),
            self.receiver.clone(),
            self.dispatcher.clone(),
        );

        match self.dispatcher.lock().as_deref_mut() {
            Ok(dispatcher) => {
                if let Err(e) = dispatcher.handle(context) {
                    eprintln!("Error: {:#?}", e);
                }
            }
            Err(e) => eprintln!("Error: {:#?}", e),
        }

        Ok(())
    }

    /// Обрабатывает входящее соединение (поток данных).
    fn handle_stream(&mut self, stream: std::net::TcpStream) -> std::io::Result<()> {
        stream.set_write_timeout(Some(Duration::from_secs(30)))?;
        stream.set_read_timeout(Some(Duration::from_secs(30)))?;

        let welsib_stream = if self.arguments.has_ssl() {
            self.ssl_manager.create_welsib_stream(stream, &self.config)
        } else {
            Some(WelsibStream {
                ssl_stream: None,
                tcp_stream: Some(stream),
            })
        };

        if let Some(welsib_stream) = welsib_stream {
            let context = WelsibContext::new(
                Some(Arc::new(Mutex::new(welsib_stream))),
                self.channels.clone(),
                self.resource.clone(),
                self.has_executor_connected.clone(),
                self.api_request_elapsed_time.clone(),
                self.sender.clone(),
                self.receiver.clone(),
                self.dispatcher.clone(),
            );

            match self.dispatcher.lock().as_deref_mut() {
                Ok(dispatcher) => {
                    if let Err(e) = dispatcher.handle(context) {
                        eprintln!("Error: {:#?}", e);
                    }
                }
                Err(e) => eprintln!("Error: {:#?}", e),
            }
        }

        Ok(())
    }
}