use super::super::WelsibContext;
use super::super::WelsibState;

impl WelsibContext {
    pub fn do_write_response(&mut self, next_state: WelsibState) {
        let output = match self.response() {
            Some(response) => {
                // println!("Response: {:#?}", response);
                Some([response.to_bytes(), b"\r\n\r\n".to_vec()].concat())
            }
            None => None,
        };

        let state = match self.stream() {
            Some(stream) => {
                match stream.lock().as_deref_mut() {
                    Ok(stream) => {
                        // TODO: добавить в заголовок цифровую подпись
                        match output {
                            Some(response_bytes) => {
                                match stream.write(&response_bytes) {
                                    Ok(()) => {
                                        // Success
                                        next_state
                                    }
                                    Err(_e) => {
                                        // TODO: WelsibState::AwaitHandleWriteError
                                        // TODO: WelsibState::AwaitWriteErrorInLog
                                        WelsibState::Done
                                    }
                                }
                            }
                            None => next_state,
                        }
                    }
                    Err(e) => {
                        eprintln!("Error do_await_read_request: {:#?}", e);
                        // TODO: WelsibState::AwaitHandleWriteError
                        WelsibState::Done
                    }
                }
            }
            None => WelsibState::Done,
        };

        self.set_state(state);
    }
}