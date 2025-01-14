use super::super::WelsibContext;
use super::super::WelsibState;
use crate::net::welsib_http_request::{RequestHeader, GeneralHeader, EntityHeader};
use crate::net::welsib_http_response::{WelsibResponseHeader, ResponseHeader};
use crate::net::WelsibHttpResponse;
use crate::conv::vec2hex;

impl WelsibContext {
    pub fn do_read_file(&mut self, error_code: Option<u16>) {
        let request = self.request();
        // println!("Request: {:#?}", &request);

        let (resource_id, has_gzip) = match request {
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
                (
                    match error_code {
                        Some(error_code) => String::from(match error_code {
                            400 => "/400.html",
                            404 => "/404.html",
                            500 => "/500.html",
                            _ => "/500.html",
                        }),
                        None => request.reqest_line.uri.clone(),
                    },
                    has_gzip,
                )
            }
            None => {
                self.set_state(WelsibState::Done);
                return;
            }
        };
        // println!("DEBUG: {:#?} {:#?}", &resource_id, &has_gzip);

        let output_bytes = match self.resource().lock().as_deref_mut() {
            Ok(resource) => {
                match if has_gzip {
                    resource.gzipped_list.get(&resource_id)
                } else {
                    resource.list.get(&resource_id)
                } {
                    Some(output_bytes) => Some(output_bytes.clone()),
                    None => None,
                }
            }
            Err(e) => {
                eprintln!("Error do_await_read_request (output_bytes): {:#?}", e);
                None
            }
        };

        let output_signature = match self.resource().lock().as_deref_mut() {
            Ok(resource) => {
                match resource.sign_list.get(&resource_id) {
                    Some(output_signature) => Some(output_signature.clone()),
                    None => None,
                }
            }
            Err(e) => {
                eprintln!("Error do_await_read_request (output_signature_bytes): {:#?}", e);
                None
            }
        };

        let next = match output_bytes {
            Some(output_bytes) => {
                let mut response = WelsibHttpResponse::new(200);
                response
                    .general_headers
                    .insert(GeneralHeader::Connection, String::from("close"));
                response
                    .response_headers
                    .insert(ResponseHeader::AcceptRanges, String::from("bytes"));
                response
                    .response_headers
                    .insert(ResponseHeader::Server, String::from("Welsib/0.1.0.0"));
                if has_gzip {
                    response
                        .entity_headers
                        .insert(EntityHeader::ContentEncoding, String::from("gzip"));
                }
                response
                    .entity_headers
                    .insert(EntityHeader::ContentLength, output_bytes.len().to_string());
                response.entity_headers.insert(
                    EntityHeader::ContentType,
                    String::from(if resource_id == String::from("/") {
                        "text/html;charset=utf-8"
                    } else {
                        match resource_id.split(".").last() {
                            Some(ext) => match ext {
                                "html" => "text/html;charset=utf-8",
                                "js" => "application/javascript;charset=utf-8",
                                "css" => "text/css;charset=utf-8",
                                "txt" => "text/plain;charset=utf-8",
                                "ico" => "image/x-icon",
                                "tgz" => "application/gzip",
                                _ => "application/octet-stream",
                            },
                            None => "text/html;charset=utf-8",
                        }
                    }),
                );

                // *-Signature: keyId="[a-zA-Z0-9_-]+",algorithm="gost3410-2018-512-with-gost3411-2012-streebog512",signature="R=[0-9a-f]{128},S=[0-9a-f]{128}"
                // *-Public-Key: (compressed,|(uncompressed,)?)X=[0-9a-f]{128}(,Y=[0-9a-f]{128})?
                // *-Curve-Parameters: "OID: 1.2.643.7.1.2.1.2.1, TC26: id-tc26-gost-3410-12-512-paramSetA"
                if let Some(output_signature) = output_signature {
                    response.extension_headers.insert(WelsibResponseHeader::XSignature.to_string(),
                    String::from("keyId=\"welsib\",algorithm=\"gost3410-2018-512-with-gost3411-2012-streebog512\",signature=\"R=") +
                        vec2hex(output_signature.to_be_bytes()[0..64].to_vec()).as_str() +
                        ",S=" +
                        vec2hex(output_signature.to_be_bytes()[64..128].to_vec()).as_str() + "\""
                    );
                    response.extension_headers.insert(WelsibResponseHeader::XPublicKey.to_string(), self.resource().lock().as_deref_mut().unwrap().verify_key.to_hex());
                    response.extension_headers.insert(WelsibResponseHeader::XCurveParameters.to_string(), String::from("\"OID: 1.2.643.7.1.2.1.2.1, TC26: id-tc26-gost-3410-12-512-paramSetA\""));
                }

                response.message_body = output_bytes.clone();
                // self.set_output(response.message_body.clone());
                self.set_response(response);
                WelsibState::AwaitWriteResponse
            }
            None => WelsibState::Done,
        };

        self.set_state(next);
    }
}