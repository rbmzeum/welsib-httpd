use super::super::WelsibContext;
use super::super::WelsibState;
use crate::api::{ApiRequest, ApiResponse, WelsibDtoInterface};
use std::time::Duration;

impl WelsibContext {
    pub fn do_await_initiator(&mut self) {
        println!("Await initiator: Begin");

        // Вернёт true, когда в канале появится сообщение о том, что в очереди появились новые соединения
        // TODO: периодически (раз в секунду или несколько миллисекунд?) будет тоже возвращать true, чтобы обновить состояние соединения клиента с исполнителем
        let has_new_connection = match self.receiver().lock().as_deref_mut() {
            Ok(receiver) => {
                println!("Await initiator: recv() // need status update or has new records");
                receiver.recv().unwrap_or(false)
            }
            _ => false,
        };
        println!("Await initiator: recv {:#?}", &has_new_connection);
        println!(
            "Await initiator: is_some_stream {:#?}",
            &self.stream().is_some()
        );
        let next_state = if has_new_connection {
            // Поступил API запрос от initiator или запрос обновления статуса от dispatcher
            if let Some(channels) = self.channels() {
                let next_state = if let Ok(channels) = channels.lock().as_deref_mut() {
                    println!("Channels len(): {:#?}", channels.len());
                    let next_state = if channels.len() > 0 {
                        // TODO: для распараллеливания и распределения запросов на разные executor-ы сделать обработку по одной записи без while
                        while let Some(channel) = channels.pop_front() {
                            println!("Channel tacked");
                            // TODO: Прочитать из канала сообщение от инициатора к исполнителю
                            if let Ok(receiver) =
                                channel.receiver_from_initiator.lock().as_deref_mut()
                            {
                                println!("Receiver from initiator: recv()");
                                match receiver.recv().as_deref_mut() {
                                    Ok(buffer) => {
                                        // ...
                                        println!("Buffer: {:?}", &buffer);
                                        let json = String::from_utf8(buffer.to_vec())
                                            .unwrap_or("".to_string());
                                        println!("Json: {:?}", &json);
                                        // json.truncate(u32::from_be_bytes(header_size) as usize);

                                        // TODO: если разорвана связь с клиентом executor-а, то вернуть данные в канал, и сам канал поместить в конец очереди и перейти в состояние Done
                                        // Обновить статус исполнителя на основе успешности или провала выполнения операции

                                        // ...
                                        match self.stream() {
                                            Some(stream) => {
                                                match stream.lock().as_deref_mut() {
                                                    Ok(stream) => {
                                                        println!("Stream begin");
                                                        let api_request =
                                                            ApiRequest::new(json.as_str());
                                                        println!("Executor request from server to client: {:#?}", &api_request);
                                                        match api_request {
                                                            Some(api_reqeust) => {
                                                                match stream.write(
                                                                    &api_reqeust
                                                                        .to_frame::<ApiRequest>(),
                                                                ) {
                                                                    Ok(()) => {
                                                                        // TODO: ждать ответа от executor-client
                                                                        // после принятия ответа пометить статус и отправить ответ в канал
                                                                        println!("Успешно отправлен запрос на executor-client, ожидание ответа...");
                                                                        if let Some(
                                                                            api_response_frame,
                                                                        ) = stream.read()
                                                                        {
                                                                            // DEBUG:
                                                                            // println!("DEBUG buffer api_response_frame: {:?}", &api_response_frame);
                                                                            let api_response = ApiResponse::from_frame(&api_response_frame);
                                                                            println!("API response from executor-client: {:#?}", &api_response);
                                                                            // executor status is enabled
                                                                            self.update_has_executor_connected(true);
                                                                            self.update_api_request_elapsed_time();
                                                                            // Release:
                                                                            match channel
                                                                                .sender_to_initiator
                                                                                .lock()
                                                                                .as_deref_mut()
                                                                            {
                                                                                Ok(sender) => {
                                                                                    match sender.send(api_response_frame) {
                                                                                        Ok(()) => {
                                                                                            println!("Успешная отпарвка API ответа в канал initiator-у");
                                                                                        },
                                                                                        Err(e) => {
                                                                                            eprintln!("Error (ошибка отправки ответа в канал для initiator ): {:#?}", e);
                                                                                        },
                                                                                    }
                                                                                    // ...
                                                                                }
                                                                                Err(e) => {
                                                                                    eprintln!("Error: {:#?}", e);
                                                                                }
                                                                            }
                                                                        } else {
                                                                            // executor status is disabled
                                                                            self.update_has_executor_connected(false);
                                                                            self.update_api_request_elapsed_time();
                                                                        }
                                                                    }
                                                                    Err(e) => {
                                                                        // Ошибка отправки executor-client-у
                                                                        // пометить статус отсутствия соедиения с исполнителем
                                                                        eprintln!("Ошибка при отправке запроса executor-client-у: {:#?}", e);
                                                                        // executor status is disabled
                                                                        self.update_has_executor_connected(false);
                                                                        self.update_api_request_elapsed_time();
                                                                    }
                                                                }
                                                            }
                                                            None => {
                                                                // TODO: неверный json от клиента
                                                                eprintln!("Ошибка: неверный json от клиента");
                                                            }
                                                        }
                                                    }
                                                    Err(e) => {
                                                        // TODO: установить статус отсуствия соединения с executor-client-ом
                                                        // добавить ошибку в лог?
                                                        // executor status is disabled
                                                        eprintln!("Нет стрима с executor-client: {:#?}", e);
                                                        self.update_has_executor_connected(false);
                                                        self.update_api_request_elapsed_time();
                                                    }
                                                };
                                            }
                                            None => {
                                                // TODO: установить статус отсуствия соединения с executor-client-ом
                                                // executor status is disabled
                                                eprintln!("WelsibStream is None");
                                                self.update_has_executor_connected(false);
                                                self.update_api_request_elapsed_time();
                                            }
                                        };
                                    }
                                    Err(e) => {
                                        eprintln!("Error: {:#?}", e);
                                    }
                                }
                            }
                        }
                        WelsibState::AwaitInitiator
                    } else {
                        println!("\n==== Отладка режима ping/pong (begin)");
                        // Поступила команда обновить статус от dispatcher
                        // ping/pong и если соединение разорвано, обновить статус и вернуть Done, иначе обновить статус и вернуть AwaitInitiator

                        // Проверить время последней отправки ответа, если оно не превышает определённого интервала, то игнорировать и не отправлять запрос
                        match self.api_request_elapsed_time().lock().as_deref_mut() {
                            Ok(api_request_elapsed_time) => {
                                match api_request_elapsed_time.elapsed() {
                                    Ok(elapsed) => {
                                        println!("Elapsed time: {:#?}", &elapsed);
                                        // ======
                                        if let Ok(has_executor_connected) = self.has_executor_connected().lock().as_deref_mut() {
                                            if elapsed < Duration::from_secs(4) && *has_executor_connected {
                                                // С момента предыдущей проверки прошло мало времени, поэтому игнорируем повторную проверку
                                                println!("С момента предыдущей проверки прошло мало времени, поэтому игнорируем повторную проверку");
                                                WelsibState::AwaitInitiator
                                            } else {
                                                println!("Отправка запроса ping для проверки статуса соединения");
                                                match self.stream() {
                                                    Some(stream) => {
                                                        match stream.lock().as_deref_mut() {
                                                            Ok(stream) => {
                                                                println!("Stream begin (ping/pong)");
                                                                let api_reqeust = ApiRequest {
                                                                    command: String::from("ping"),
                                                                    attributes: String::from(""),
                                                                };
                                                                println!(
                                                                    "Executor request from server to client: {:#?}",
                                                                    &api_reqeust
                                                                );
                                                                match stream.write(&api_reqeust.to_frame::<ApiRequest>()) {
                                                                    Ok(()) => {
                                                                        // TODO: ждать ответа от executor-client
                                                                        // после принятия ответа пометить статус и отправить ответ в канал
                                                                        println!("Успешно отправлен запрос на executor-client, ожидание ответа...");
                                                                        if let Some(api_response_frame) = stream.read() {
                                                                            // DEBUG:
                                                                            // println!("DEBUG buffer api_response_frame: {:?}", &api_response_frame);
                                                                            let api_response = ApiResponse::from_frame(
                                                                                &api_response_frame,
                                                                            );
                                                                            // TODO: проверить, что действительно вернулся именно "pong"
                                                                            println!(
                                                                                "API response from executor-client: {:#?}",
                                                                                &api_response
                                                                            );
                                                                            // executor status is enabled
                                                                            // if let Ok(has_executor_connected) = self
                                                                            //     .has_executor_connected
                                                                            //     .lock()
                                                                            //     .as_deref_mut()
                                                                            // {
                                                                            //     *has_executor_connected = true;
                                                                            // }
                                                                            WelsibState::AwaitInitiator
                                                                        } else {
                                                                            eprintln!("Stream read from executor-client is empty");
                                                                            // executor status is disabled
                                                                            // if let Ok(has_executor_connected) = self
                                                                            //     .has_executor_connected
                                                                            //     .lock()
                                                                            //     .as_deref_mut()
                                                                            // {
                                                                            //     *has_executor_connected = false;
                                                                            // }
                                                                            WelsibState::Done
                                                                        }
                                                                    }
                                                                    Err(e) => {
                                                                        // Ошибка отправки executor-client-у
                                                                        // пометить статус отсутствия соедиения с исполнителем
                                                                        eprintln!("Ошибка при отправке запроса executor-client-у: {:#?}", e);
                                                                        // executor status is disabled
                                                                        // if let Ok(has_executor_connected) = self
                                                                        //     .has_executor_connected
                                                                        //     .lock()
                                                                        //     .as_deref_mut()
                                                                        // {
                                                                        //     *has_executor_connected = false;
                                                                        // }
                                                                        WelsibState::Done
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => {
                                                                eprintln!("Ошибка доступа к стриму при выполнении команды ping: {:#?}", e);
                                                                // executor status is disabled
                                                                // if let Ok(has_executor_connected) =
                                                                //     self.has_executor_connected.lock().as_deref_mut()
                                                                // {
                                                                //     *has_executor_connected = false;
                                                                // }
                                                                WelsibState::Done
                                                            }
                                                        }
                                                    }
                                                    None => {
                                                        eprintln!("Ошибка доступа к стриму при выполнении команды ping (нет стрима)");
                                                        // executor status is disabled
                                                        // if let Ok(has_executor_connected) =
                                                        //     self.has_executor_connected.lock().as_deref_mut()
                                                        // {
                                                        //     *has_executor_connected = false;
                                                        // }
                                                        WelsibState::Done
                                                    }
                                                }
                                            }
                                        } else {
                                            // Не удалось обработать в этот раз, передаём управление на следующую итерацию
                                            println!("Не удалось обработать в этот раз, передаём управление на следующую итерацию");
                                            WelsibState::AwaitInitiator
                                        }
                                        // ======

                                        // if elapsed < Duration::from_secs(3) { // & has_executor_connected is true
                                        // if false {
                                        //     // TODO: если не прошло достаточно времени с момента последней операции
                                        //     // считать что соединение не разорвано и продолжать приём сообщений от инициатора
                                        //     WelsibState::AwaitInitiator
                                        // } else {
                                            
                                        // }
                                    },
                                    Err(e) => {
                                        // Проблема с elapsed()
                                        eprintln!("Проблема с context.api_request_elapsed_time.elapsed() {:#?}", e);
                                        WelsibState::Done
                                    },
                                }
                            },
                            Err(e) => {
                                // Проблема с доступом к переменной, возможно ошибка программиста
                                eprintln!("Проблема с доступом к переменной, возможно ошибка программиста: {:#?}", e);
                                WelsibState::Done
                            },
                        }
                    };
                    next_state
                } else {
                    // Не удалось получить доступ к очереди каналов
                    println!("Не удалось получить доступ к очереди каналов");
                    WelsibState::Done
                };

                // WelsibState::AwaitInitiator
                next_state
            } else {
                // Ситуация, в которой запустился executor-server, но при этом не может получать данные по каналам от initiator-server всвязи с отстуствием очереди каналов
                WelsibState::AwaitInitiator
            }
        } else {
            WelsibState::AwaitInitiator
        };

        if next_state == WelsibState::AwaitInitiator {
            self.update_has_executor_connected(true);
            self.update_api_request_elapsed_time();
        } else {
            self.update_has_executor_connected(false);
            self.update_api_request_elapsed_time();
        }

        self.set_state(next_state);
    }
}
