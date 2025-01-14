// use super::super::WelsibContext;
// use super::super::WelsibState;

// impl WelsibContext {
//     pub fn do_handshake(&mut self) {
//         let next_state = match self.stream() {
//             Some(stream) => {
//                 match stream.lock().as_deref_mut() {
//                     Ok(stream) => {
//                         match stream.read() {
//                             Some(res) => {
//                                 let status = StatusDto::from_frame(&res);
//                                 if let Some(status) = status {
//                                     if status.is_ready() {
//                                         let request = HandshakeRequest::new();
//                                         match stream.write(&request.to_frame::<HandshakeRequest>())
//                                         {
//                                             Ok(()) => {
//                                                 let handshake_buffer = stream.read(); // TODO: сделать способ определить что соединение зависло или разорвано и сменить статус в соотвествии с этим, если не произошло восстановления соединения
//                                                 match handshake_buffer {
//                                                     Some(b) => {
//                                                         let res = HandshakeResponse::from_frame(&b);
//                                                         // println!("Handshake response: {:#?}", &res);
//                                                         if let Some(response) = res {
//                                                             let verify_key = esig::Point {
//                                                                 x: num_bigint::BigInt::parse_bytes(
//                                                                     VERIFY_KEY.0,
//                                                                     16,
//                                                                 )
//                                                                 .unwrap(),
//                                                                 y: num_bigint::BigInt::parse_bytes(
//                                                                     VERIFY_KEY.1,
//                                                                     16,
//                                                                 )
//                                                                 .unwrap(),
//                                                             };
//                                                             if response.verify(
//                                                                 &slice2vec(request.random),
//                                                                 &verify_key,
//                                                             ) {
//                                                                 // Success
//                                                                 println!("Handshake success");
//                                                                 // executor status is enabled
//                                                                 // self.update_has_executor_connected(true);
//                                                                 // self.update_api_request_elapsed_time();
//                                                                 WelsibState::AwaitInitiator
//                                                             } else {
//                                                                 WelsibState::Done
//                                                             }
//                                                         } else {
//                                                             WelsibState::Done
//                                                         }
//                                                     }
//                                                     None => WelsibState::Done,
//                                                 }
//                                             }
//                                             Err(_e) => {
//                                                 // TODO: WelsibState::AwaitHandleWriteError
//                                                 // TODO: WelsibState::AwaitWriteErrorInLog
//                                                 eprintln!("Error");
//                                                 WelsibState::Done
//                                             }
//                                         }
//                                     } else {
//                                         WelsibState::Done
//                                     }
//                                 } else {
//                                     WelsibState::Done
//                                 }
//                             }
//                             None => {
//                                 eprintln!("Error: Client not ready");
//                                 WelsibState::Done
//                             }
//                         }
//                     }
//                     Err(e) => {
//                         eprintln!("Error do_await_read_request: {:#?}", e);
//                         // TODO: WelsibState::AwaitHandleWriteError
//                         WelsibState::Done
//                     }
//                 }
//             }
//             None => WelsibState::Done,
//         };

//         if next_state == WelsibState::AwaitInitiator {
//             self.update_has_executor_connected(true);
//             self.update_api_request_elapsed_time();
//         } else {
//             self.update_has_executor_connected(false);
//             self.update_api_request_elapsed_time();
//         }

//         self.set_state(next_state);
//     }
// }