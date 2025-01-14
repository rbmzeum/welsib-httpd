use super::super::WelsibContext;
use super::super::WelsibState;
use crate::net::welsib_http_request::{GeneralHeader, RequestMethod};

impl WelsibContext {
    pub fn do_read_request(&mut self) {
        let available_static_file_uri = self.resource().lock().as_deref().unwrap().available_static_file_uri.clone(); // FIXME: перенести в 
        let input_bytes = match self.stream() {
            Some(stream) => match stream.lock().as_deref_mut() {
                Ok(stream) => stream.read(),
                Err(e) => {
                    eprintln!("Error do_await_read_request: {:#?}", e);
                    None
                }
            },
            None => None,
        };

        match input_bytes {
            Some(input_bytes) => {
                self.set_input(input_bytes);
                let request = self.request();
                match request {
                    Some(request) => {
                        // println!("Request: {:#?}", request);
                        // Маршрутизация состояний на основе информации из запроса
                        let is_activator = if let Some(header_connection) =
                            request.general_headers.get(&GeneralHeader::Connection)
                        {
                            if header_connection.to_lowercase().trim().eq("upgrade") {
                                if let Some(header_upgrade) =
                                    request.general_headers.get(&GeneralHeader::Upgrade)
                                {
                                    if header_upgrade.to_lowercase().trim().eq("welsib/0.1.0.0") {
                                        // Connection: upgrade
                                        // Upgrade: welsib/0.1.0.0
                                        true
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if request.reqest_line.method == RequestMethod::GET && !is_activator {
                            // Перейти на шаг отправки статического файла или на шаг отправки сообщения о 404 ошибке
                            if available_static_file_uri.contains(&request.reqest_line.uri) { // FIXME: этот метод contains возможно не работает так как надо
                                self.set_state(WelsibState::AwaitReadFile);
                            } else {
                                self.set_state(WelsibState::AwaitRead404File);
                            }

                        } else if request.message_body.len() > 0
                            && !is_activator
                            && (request.reqest_line.method == RequestMethod::POST
                                    && request.reqest_line.uri.as_str() == "/api")
                        {
                            // Перейти на шаг проверки доступности API или ACTIVATE
                            self.set_state(WelsibState::AwaitExecutor);
                        // } else if !is_activator && (request.reqest_line.uri.as_str() == "/payment" || request.reqest_line.uri.as_str() == "/receipt") {
                        //     // payment - уведомления о платежах
                        //     // receipt - уведомления о фискализации
                        //     self.set_state(WelsibState::AwaitPaymentNotification);
                        } else if is_activator {
                            self.set_state(WelsibState::AwaitUpgrade);
                        } else {
                            // println!("DEBUG Request: {:#?}", &request);
                            self.set_state(WelsibState::AwaitRead400File);
                        }
                    }
                    None => {
                        self.set_state(WelsibState::Done);
                    }
                };
            }
            None => {
                // Не удалось считать с потока данные запроса
                self.set_state(WelsibState::AwaitRead500File);
            }
        };
    }
}