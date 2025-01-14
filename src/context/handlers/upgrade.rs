use super::super::WelsibContext;
use super::super::WelsibState;
use crate::net::welsib_http_request::GeneralHeader;
use crate::net::WelsibHttpResponse;

impl WelsibContext {
    pub fn do_upgrade(&mut self) {
        let request = self.request();
        // println!("Request: {:#?}", &request);

        // Код подготовки ответа для welsib-dynamics и перехода в состояние постоянного соединения на основе протокола welsib/0.1.0.0 (альтернатива websocket)

        // request:
        // GET / HTTP/1.1
        // Host: welsib.ru
        // Connection: upgrade
        // Upgrade: welsib/0.1.0.0

        let next_state = match request {
            Some(request) => {
                let protocol = request.general_headers.get(&GeneralHeader::Upgrade);
                match protocol {
                    Some(protocol) => {
                        match protocol.as_str() {
                            "welsib/0.1.0.0" => {
                                // response:
                                // HTTP/1.1 101 Switching Protocols
                                // Upgrade: welsib/0.1.0.0
                                // Connection: upgrade

                                let mut response = WelsibHttpResponse::new(101);
                                response
                                    .general_headers
                                    .insert(GeneralHeader::Upgrade, String::from("welsib/0.1.0.0"));
                                response
                                    .general_headers
                                    .insert(GeneralHeader::Connection, String::from("upgrade"));
                                self.set_response(response);

                                // Успеное завершение состояния
                                WelsibState::AwaitWrite101Response
                            }
                            _ => {
                                // Неподдерживаемый протокол и/или его версия
                                WelsibState::AwaitRead400File
                            }
                        }
                    }
                    None => {
                        // Нет заголовка Upgrade с назвнием и версией протокола
                        WelsibState::AwaitRead400File
                    }
                }
            }
            None => {
                // Нет данных о запросе
                WelsibState::AwaitRead400File
            }
        };

        self.set_state(next_state);
    }
}