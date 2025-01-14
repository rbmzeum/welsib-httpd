use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use crate::net::WelsibStream;
use crate::resource::WelsibResource;
use crate::dispatcher::Dispatcher;
use crate::channel::WelsibChannel;
use crate::context::WelsibState;
use crate::net::WelsibHttpRequest;
use crate::net::WelsibHttpResponse;
use std::time::SystemTime;
use std::sync::mpsc::{Sender, Receiver};

/// Контекст для обработки запросов.
pub struct WelsibContext {
    state: WelsibState, // Состояние контекста
    // executors: Arc<Mutex<HashMap<ThreadId, Thread>>>, // TODO: через диспетчер список доступных executor-ов которым можно отправить задачу на выполнение
    has_executor_connected: Arc<Mutex<bool>>, // Статус наличия соединения executor (процесс на фронт-хосте) с клиентом (микросервис обработки API команд на бекенд-хосте)
    api_request_elapsed_time: Arc<Mutex<SystemTime>>, // Время последнего успешного ответа от executor-client
    stream: Option<Arc<Mutex<WelsibStream>>>, // TcpStream или SslStream<TcpStream>
    resource: Arc<Mutex<WelsibResource>>,     // статические данные
    input: Vec<u8>,                           // сырые данные поступившие от клиента
    request: Option<WelsibHttpRequest>, // сформированный объект запроса на основе данных из self.input
    response: Option<WelsibHttpResponse>, // сформированный объект ответа, который может менять по мере передачи контекста между обработчиками
    sender: Arc<Mutex<Sender<bool>>>, // основной канал по которому от диспетчера процессу-исполнителю поступают команды, когда в очереди появляются записи
    receiver: Arc<Mutex<Receiver<bool>>>, // основной канал по которому исполнитель получает команды от диспетчера, о наличии в очереди новых записей
    channels: Option<Arc<Mutex<VecDeque<WelsibChannel>>>>, // каналы создаваемые Initiator-ами для Executor-ов
    dispatcher: Arc<Mutex<Dispatcher>>,                    // доступ к диспетчеру из контекста
}

impl WelsibContext {
    /// Создает новый контекст.
    pub fn new(
        stream: Option<Arc<Mutex<WelsibStream>>>,
        channels: Option<Arc<Mutex<VecDeque<WelsibChannel>>>>,
        resource: Arc<Mutex<WelsibResource>>,
        has_executor_connected: Arc<Mutex<bool>>,
        api_request_elapsed_time: Arc<Mutex<SystemTime>>,
        sender: Arc<Mutex<Sender<bool>>>,
        receiver: Arc<Mutex<Receiver<bool>>>,
        dispatcher: Arc<Mutex<Dispatcher>>,
    ) -> Self {
        Self {
            state: WelsibState::AwaitBegin,
            has_executor_connected,
            api_request_elapsed_time,
            stream,
            resource,
            input: vec![],
            request: None,
            response: None,
            sender,
            receiver,
            channels,
            dispatcher,
        }
    }

    pub fn state(&self) -> WelsibState {
        self.state
    }

    pub fn stream(&self) -> &Option<Arc<Mutex<WelsibStream>>> {
        &self.stream
    }

    pub fn resource(&self) -> &Arc<Mutex<WelsibResource>> {
        &self.resource
    }

    pub fn dispatcher(&self) -> &Arc<Mutex<Dispatcher>> {
        &self.dispatcher
    }

    pub fn has_executor_connected(&self) -> &Arc<Mutex<bool>> {
        &self.has_executor_connected
    }

    pub fn update_has_executor_connected(&self, value: bool) {
        if let Ok(has_executor_connected) =
            self.has_executor_connected
                .lock()
                .as_deref_mut()
        {
            *has_executor_connected = value;
        }
    }

    pub fn api_request_elapsed_time(&self) -> &Arc<Mutex<SystemTime>> {
        &self.api_request_elapsed_time
    }

    pub fn update_api_request_elapsed_time(&self) {
        if let Ok(api_request_elapsed_time) = self.api_request_elapsed_time.lock().as_deref_mut() {
            *api_request_elapsed_time = SystemTime::now();
        }
    }

    pub fn sender(&self) -> &Arc<Mutex<Sender<bool>>> {
        &self.sender
    }

    pub fn receiver(&self) -> &Arc<Mutex<Receiver<bool>>> {
        &self.receiver
    }

    pub fn channels(&self) -> &Option<Arc<Mutex<VecDeque<WelsibChannel>>>> {
        &self.channels
    }

    pub fn input_bytes(&self) -> &Vec<u8> {
        &self.input
    }

    pub fn input(&self) -> String {
        String::from_utf8(self.input.to_owned())
            .unwrap_or("".to_string())
            .trim_matches('\0')
            .to_string()
    }

    pub fn request(&mut self) -> &mut Option<WelsibHttpRequest> {
        if self.request.is_none() {
            self.request = WelsibHttpRequest::from_string(self.input());
        }
        &mut self.request
    }

    pub fn response(&mut self) -> &Option<WelsibHttpResponse> {
        &self.response
    }

    pub fn set_response(&mut self, new_response: WelsibHttpResponse) {
        self.response = Some(new_response)
    }

    pub fn set_state(&mut self, new_state: WelsibState) {
        println!("New state: {:#?}", &new_state);
        self.state = new_state
    }

    pub fn set_input(&mut self, new_input: Vec<u8>) {
        self.input = new_input
    }
}
