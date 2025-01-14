use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
};

pub struct WelsibChannel {
    pub sender_to_executor: Arc<Mutex<Sender<Vec<u8>>>>,
    pub receiver_from_initiator: Arc<Mutex<Receiver<Vec<u8>>>>,
    pub sender_to_initiator: Arc<Mutex<Sender<Vec<u8>>>>,
    pub receiver_from_executor: Arc<Mutex<Receiver<Vec<u8>>>>,
    // TODO: добавить статусы соединений initiator с клиентом и executor с клиентом
    // pub status_initiator: Arc<Mutex<StatusInitiator>>,
    // Initiator statuses:
    // 1. Получил данные от клиента, передал данные исполнителю и перешёл в состояние ожидания ответа
    // 2. Клиент разорвал соединение не дождавшись ответа (пометить статус для исполнителя, что инициатор потерял связь с клиентом, переход в Done)
    // 3. Получил ответ исполнителя и отправил его клиенту (переход в Done)
    // 4. Ответил клиенту, что в данный момент времени исполнитель не доступен
    //
    // pub status_executor: Arc<Mutex<StatusExecutor>>,
    // Executor statuses:
    // 1. Ожидает появления в очереди записей
    // 2. Проверить статус инициатора, если инициатор всё ещё на связи, то извлеч канал связи из очереди и считать данные из него, проверить статус клиента исполнителя, если клиент на связи, то отправить команду и ждать ответа
    // 2.1. Если связь клиента с инициатором разорвана, то извречь канал связи из очереди и перейти к обработке следующей записи или в статус появления в очереди записей
    // 2.2. Если связь клиента с исполниетлем разорвана и при этом связь с инициатором не разорвана, то (сначала вернуть в очередь канал связи, записав в него сообщение, увеличить на единицу attemts) ответить клиенту инициатора сообщением о недоступности исполнителя после нескольких попыток связаться с клиентом исполнителя
    // 3. Приняв ответ клиента исполнителя проверить статус инициатора, если инициатор всё ещё на связи, то записать в канал ответ от клиента исполнителя и перейти в состояние Done
    // 3.1. Если связь клиента с инициатором разорвана, то перейти в состояние Done
}

impl WelsibChannel {
    pub fn new() -> Self {
        let (sender_to_executor, receiver_from_initiator) = channel();
        let (sender_to_initiator, receiver_from_executor) = channel();

        let sender_to_executor = Arc::new(Mutex::new(sender_to_executor));
        let receiver_from_initiator = Arc::new(Mutex::new(receiver_from_initiator));
        let sender_to_initiator = Arc::new(Mutex::new(sender_to_initiator));
        let receiver_from_executor = Arc::new(Mutex::new(receiver_from_executor));

        Self {
            sender_to_executor,
            receiver_from_initiator,
            sender_to_initiator,
            receiver_from_executor,
        }
    }
}
