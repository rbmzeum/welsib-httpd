use crate::context::{WelsibContext, WelsibState};
use std::collections::HashMap;
use std::io::Error;
use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle, Thread};
use std::{
    collections::VecDeque,
    thread::{sleep, spawn, ThreadId},
    time::Duration,
};

pub struct Dispatcher {
    handlers: Arc<Mutex<VecDeque<JoinHandle<Result<(), Error>>>>>,
    threads: Arc<Mutex<HashMap<ThreadId, Thread>>>,
}

impl Dispatcher {
    pub fn new() -> Self {
        let handlers = Arc::new(Mutex::new(VecDeque::<JoinHandle<Result<(), Error>>>::new()));
        let hndlrs = handlers.clone();
        let threads = Arc::new(Mutex::new(HashMap::new()));
        let thrds = threads.clone();
        let dispatcher = spawn(move || -> std::io::Result<()> {
            loop {
                // println!("LOOP");
                match hndlrs.lock().as_deref_mut() {
                    Ok(handlers) => {
                        while handlers.len() > 0 {
                            // println!("Handlers len: {}", handlers.len());
                            let _ = match handlers.pop_back() {
                                Some(handler) => {
                                    let thread = handler.thread();
                                    // let name = thread.name();
                                    match thrds.lock().as_deref_mut() {
                                        Ok(threads) => {
                                            threads.insert(thread.id(), thread.to_owned());
                                            // FIXME: надо чистить массив от выполнившихся тредов
                                        }
                                        Err(e) => {
                                            eprintln!("Error threads: {:#?}", e);
                                        }
                                    };
                                    // println!("Thread id: {:#?}", &thread.id());
                                    // let x = std::thread::current().id();
                                    // let x = thread.id();
                                }
                                None => {
                                    eprintln!(" Warning: не удалось извлечь обработчик");
                                    // Ok(())
                                }
                            };
                        }
                    }
                    Err(e) => {
                        eprintln!("Error [handlers.push_front(handler)]: {:#?}", e);
                    }
                };

                sleep(Duration::from_millis(1));
            }
        });

        let _dispatcher_thread = dispatcher.thread();
        // println!("Dispatcher thread: {:#?}", dispatcher_thread);

        Self { handlers, threads }
    }

    pub fn handle(&mut self, context: WelsibContext) -> std::io::Result<()> {
        let context = Arc::new(Mutex::new(context));
        let threads = self.threads.clone();
        let handler = spawn(move || -> std::io::Result<()> {
            match context.lock().as_deref_mut() {
                Ok(context) => {
                    // println!("Handle context: {:#?}", &context.state());
                    while context.state() != WelsibState::Done {
                        Self::dispatch(context);
                        // TODO: если context.stream() отсоединился, то завершить цикл
                        // обработать context.stream().take_error(), и при необходимости выйти из цикла
                    }
                }
                Err(e) => {
                    eprintln!("Error (handle request): {:#?}", e);
                }
            };

            match threads.lock().as_deref_mut() {
                Ok(threads) => {
                    threads.remove(&std::thread::current().id()); // TODO: сохранять так же время запуска процесса, и выполнять стратегию удаления процесса из списка, если связь потеряна или протокол требует close, так же процесс может завершиться операционной системой принудительно и не дойти до этого места очистки, надо учесть и это
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Error {:#?}", &e);
                    Ok(()) // FIXME: вернуть Error
                }
            }
        });

        match self.handlers.lock().as_deref_mut() {
            Ok(handlers) => {
                handlers.push_front(handler);
                // println!("Pushed handler: {}", handlers.len());
            }
            Err(e) => {
                eprintln!("Error [handlers.push_front(handler)]: {:#?}", e);
            }
        };

        Ok(())
    }

    pub fn threads(&self) -> &Arc<Mutex<HashMap<ThreadId, Thread>>> {
        &self.threads
    }

    fn dispatch(context: &mut WelsibContext) {
        // TODO: определить по заголовку, если в заголовке Upgrade: welsib-activator/1, то подключить обработку этого протокола
        // Типы запросов:
        match context.state() {
            // Ограничение на размер запроса 4096 байт.
            WelsibState::AwaitBegin => context.do_begin(), // начальный роутинг, разделяет web и system
            WelsibState::AwaitUpdateExecutorStatus => context.do_update_executor_status(), // системный обработчик, посылает периодически в канал сигнал для обновления статуса
            WelsibState::AwaitReadRequest => context.do_read_request(), // начальный web маршрутизация на основе входящего сетевого запроса
            WelsibState::AwaitReadFile => context.do_read_file(None),
            WelsibState::AwaitRead400File => context.do_read_file(Some(400)),
            WelsibState::AwaitRead404File => context.do_read_file(Some(404)),
            WelsibState::AwaitRead500File => context.do_read_file(Some(500)),
            WelsibState::AwaitWriteResponse => context.do_write_response(WelsibState::Done),
            WelsibState::AwaitUpgrade => context.do_upgrade(),
            WelsibState::AwaitWrite101Response => {
                context.do_write_response(WelsibState::AwaitHandshake)
            }
            // WelsibState::AwaitHandshake => context.do_handshake(),
            WelsibState::AwaitExecutor => context.do_await_executor(),
            // WelsibState::AwaitPaymentNotification => context.do_await_payment_notification(),
            WelsibState::AwaitInitiator => context.do_await_initiator(),
            // TODO: здесь перечислить обработчики
            _ => {
                eprintln!("Unknown WelsibState");
                context.set_state(WelsibState::Done);
            }
        };
    }
}
