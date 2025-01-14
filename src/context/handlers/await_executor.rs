use super::super::WelsibContext;
use super::super::WelsibState;
use crate::channel::WelsibChannel;
use crate::net::welsib_http_request::{RequestHeader, EntityHeader, GeneralHeader};
use crate::net::welsib_http_response::ResponseHeader;
use crate::net::WelsibHttpResponse;
use crate::helpers::compress;
use crate::api::{ApiResponse, WelsibDtoInterface};

impl WelsibContext {
    pub fn do_await_executor(&mut self) {
        println!("Await executor: begin");
        let has_executor_connected = self.has_executor_connected().clone();
        let request = self.request();
        let channel = WelsibChannel::new();
        // println!("Request: {:#?}", &request);
        println!("Status (has_executor_connected): {:#?}", &has_executor_connected);

        let has_gzip = match request {
            Some(request) => {
                let has_gzip = if let Some(content_encoding) =
                    request.request_headers.get(&RequestHeader::AcceptEncoding)
                {
                    if content_encoding.contains("gzip") {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                has_gzip
            }
            None => {
                self.set_state(WelsibState::Done);
                return;
            }
        };

        // проверить статус соединения исполнителя-сервера с исполнителем-клиентом
        println!("Has executor connected: {:#?}", &has_executor_connected);
        if let Ok(has_executor_connected) = has_executor_connected.lock().as_deref_mut() {
            if !*has_executor_connected {
                eprintln!("Executor client not connected");
                // TODO: ответить 500 ошибкой (или новым кодом о том, что ресурс временно не доступен, зайдите позднее)
                // let api_response = ApiResponse::from_resource(ERROR_500_STATIC_JSON); // TODO: создать статические ответы об ошибках с подписями и загружать их в ресурсы
                // let buffer = load_from_resource_json_error();
                // let api_response = ApiResponse::from_frame(&buffer.to_vec());
                // let next_status = if let Some(api_response) = api_response {
                //     let bytes = api_response.to_json::<ApiResponse>().as_bytes().to_vec();
                //     let mut http_response = WelsibHttpResponse::new(500);
                //     http_response
                //         .general_headers
                //         .insert(GeneralHeader::Connection, String::from("close"));
                //     http_response
                //         .response_headers
                //         .insert(ResponseHeader::AcceptRanges, String::from("bytes"));
                //     if has_gzip {
                //         http_response
                //             .entity_headers
                //             .insert(EntityHeader::ContentEncoding, String::from("gzip"));
                //     }
                //     http_response.entity_headers.insert(
                //         EntityHeader::ContentType,
                //         String::from("application/octet-stream"),
                //     );

                //     if has_gzip {
                //         match compress(&bytes) {
                //             Ok(compressed_bytes) => {
                //                 http_response
                //                 .entity_headers
                //                 .insert(EntityHeader::ContentLength, compressed_bytes.len().to_string());
                //                 http_response.message_body = compressed_bytes;
                //             },
                //             Err(e) => {
                //                 eprintln!("Compress error: {:#?}", e);
                //             },
                //         }
                //     } else {
                //         http_response
                //             .entity_headers
                //             .insert(EntityHeader::ContentLength, bytes.len().to_string());
                //         http_response.message_body = bytes;
                //     }
                //     self.set_response(http_response);

                //     // Успеное завершение состояния
                //     WelsibState::AwaitWriteResponse
                // } else {
                //     WelsibState::Done
                // };

                let next_status = WelsibState::Done; // DEBUG (определяется выше в закомментированном коде, убрать когда там будет сделано)
                self.set_state(next_status);
                return;
            }
        }

        let message_body = if let Some(request) = request {
            Some(request.message_body.clone())
        } else {
            None
        };

        let sender_to_executor = channel.sender_to_executor.clone();
        let receiver_from_executor = channel.receiver_from_executor.clone();

        // Запись канала связи в очередь
        println!("Запись канала связи в очередь");
        if let Some(channels) = self.channels() {
            match channels.lock().as_deref_mut() {
                Ok(channels) => {
                    channels.push_back(channel);
                }
                Err(e) => {
                    eprintln!("Error: {:#?}", e);
                }
            }

            if let Ok(channels) = channels.lock().as_deref_mut() {
                println!("Channels len() from initiator: {:#?}", channels.len());
            }
        }

        // Отправка сигнала о том, что очередь не пуста
        println!("Отправка сигнала о том, что очередь не пуста");
        if let Ok(sender) = self.sender().lock().as_deref_mut() {
            match sender.send(true) {
                Ok(()) => {
                    println!(
                        "Success send from initiator to executor: сигнал, что очередь не пуста"
                    );
                }
                Err(e) => {
                    eprintln!("Error: {:#?}", e);
                }
            }
        }

        // TODO:
        // 0. Проверить статус подключения исполнителя, если не подключен, то не пытаться создать с ним канал связи, а завершить сообщением о временной недоступности сервиса
        // 1. read request from client
        // 2. send in channel to executor
        // 3. await answer from executor
        // 4. create response and next status

        // Отправка запроса исполнителю
        println!("Отправка запроса исполнителю");
        if let Some(message_body) = message_body {
            match sender_to_executor.lock().as_deref_mut() {
                Ok(sender) => {
                    match sender.send(message_body) {
                        Ok(()) => {
                            println!("Success send from initiator to executor: данные");
                            // TODO: чтение ответа от executor-server
                            println!("Чтение ответа от executor-server: begin...");
                            match receiver_from_executor.lock().as_deref_mut() {
                                Ok(receiver_from_executor) => {
                                    match receiver_from_executor.recv().as_deref_mut() {
                                        Ok(buffer) => {
                                            println!("Прочитано {:#?} байт", buffer.len());
                                            let api_response =
                                                ApiResponse::from_frame(&buffer.to_vec());
                                            let next_state = if let Some(api_response) =
                                                api_response
                                            {
                                                let bytes = api_response
                                                    .to_json::<ApiResponse>()
                                                    .as_bytes()
                                                    .to_vec();
                                                let mut http_response =
                                                    WelsibHttpResponse::new(200);
                                                http_response.general_headers.insert(
                                                    GeneralHeader::Connection,
                                                    String::from("close"),
                                                );
                                                http_response.response_headers.insert(
                                                    ResponseHeader::AcceptRanges,
                                                    String::from("bytes"),
                                                );
                                                if has_gzip {
                                                    http_response.entity_headers.insert(
                                                        EntityHeader::ContentEncoding,
                                                        String::from("gzip"),
                                                    );
                                                }
                                                http_response.entity_headers.insert(
                                                    EntityHeader::ContentType,
                                                    String::from("application/octet-stream"),
                                                );

                                                if has_gzip {
                                                    match compress(&bytes) {
                                                        Ok(compressed_bytes) => {
                                                            http_response.entity_headers.insert(
                                                                EntityHeader::ContentLength,
                                                                compressed_bytes.len().to_string(),
                                                            );
                                                            http_response.message_body =
                                                                compressed_bytes;
                                                        }
                                                        Err(e) => {
                                                            eprintln!("Compress error: {:#?}", e);
                                                        }
                                                    }
                                                } else {
                                                    http_response.entity_headers.insert(
                                                        EntityHeader::ContentLength,
                                                        bytes.len().to_string(),
                                                    );
                                                    http_response.message_body = bytes;
                                                }
                                                self.set_response(http_response);

                                                // Успеное завершение состояния
                                                println!("Success: Установлен response и перевод состояния в AwaitWriteResponse");
                                                WelsibState::AwaitWriteResponse
                                            } else {
                                                // TODO: ответить 500 ошибкой с json в котором расшифровка (например, нет соединения с executor-client)
                                                eprintln!("Error: переход в состояние Done (ошибка 500)");
                                                WelsibState::Done
                                            };

                                            // Завершение состояния
                                            self.set_state(next_state);
                                            return;
                                        }
                                        Err(e) => {
                                            eprintln!("Error: {:#?}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Error: {:#?}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error {:#?}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error {:#?}", e);
                }
            }
        }

        // TODO: Чтение ответа от исполнителя и подготовка response для отправки клиенту

        // Ждать когда в канале появится сообщение о том, что в очереди появились новые соединения
        // let has_new_connection = match self.receiver().lock().as_deref_mut() {
        //     Ok(receiver) => {
        //         receiver.recv().unwrap_or(false)
        //     },
        //     _ => false,
        // };
        // if has_new_connection {
        //     // TODO: обработать очередь и снова перейти в состояние ожидания сообщения о том, что в очереди появились записи
        //     if let Ok(channels) = self.channels().lock().as_deref_mut() {
        //         // for channel in channels {
        //         //     let res = channel.rx.recv_timeout(Duration::from_millis(100)).unwrap_or(String::from(""));
        //         //     // ...
        //         // }
        //         match self.channels().lock().as_deref_mut() {
        //             Ok(channels) => {
        //                 while let Some(channel) = channels.pop_front() {
        //                     // ...
        //                 }
        //             },
        //             _ => {},
        //         }
        //     }
        // };

        self.set_state(WelsibState::Done);
    }
}
