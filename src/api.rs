use crate::checksum::crc32;
// use crate::conv::{
//     bigint2vec::bigint2vec, slice2vec::slice2vec, vec2bigint::vec2bigint, vec2slice::vec2slice,
// };
// use crate::welsib::protocols::welsib::dto::WelsibDtoInterface;
// use esig::Point;
// use esig::Signature;
// use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use crate::welsib::*;
use crate::point::Point;
use crate::signature::Signature;
use crate::conv::{slice2vec, vec2bigint};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiRequest {
    pub command: String,    // command
    pub attributes: String, // json
}

impl ApiRequest {
    pub fn new(json: &str) -> Option<Self> {
        match serde_json::from_str::<Self>(json) {
            Ok(request) => Some(request),
            _ => None,
        }
    }

    pub fn from_frame(frame: &Vec<u8>) -> Option<Self> {
        // аутентификация
        let header_size: [u8; 4] = frame[0..4].try_into().unwrap_or_default();
        let cs = crc32(&header_size.to_vec());
        let checksum: [u8; 4] = frame[4..8].try_into().unwrap_or_default();
        if u32::from_be_bytes(checksum) != cs {
            // контрольная сумма не верна
            None
        } else {
            // контрольная сумма верна
            let json_bytes = frame[8..].to_vec();
            let mut json = String::from_utf8(json_bytes).unwrap_or("".to_string());
            json.truncate(u32::from_be_bytes(header_size) as usize);
            match serde_json::from_str::<Self>(json.as_str()) {
                Ok(request) => Some(request),
                _ => None,
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiResponse {
    pub attributes: String,    // json
    pub signature_r: [u64; 8], // 512 bit
    pub signature_s: [u64; 8], // 512 bit
    pub checksum: u32,
}

impl ApiResponse {
    // pub fn make(attributes: String, private_key: &BigInt) -> Self {
    //     let hash = esig::hash::hash(&attributes.as_bytes().to_vec());
    //     let esig::Signature { r, s } = esig::sign(&hash, private_key); // FIXME: веб-сервер не умеет подписывать, подпись ставится внешним клиентом
    //     // println!("Signature: {:#?} {:#?}", &r, &s);

    //     let signature_r = bigint2vec(r);
    //     let signature_s = bigint2vec(s);
    //     let checksum = crc32(
    //         &[
    //             attributes.clone().as_bytes().to_vec(),
    //             signature_r.clone(),
    //             signature_s.clone(),
    //         ]
    //         .concat(),
    //     );

    //     Self {
    //         attributes,
    //         signature_r: vec2slice(signature_r.clone()),
    //         signature_s: vec2slice(signature_s.clone()),
    //         checksum,
    //     }
    // }

    pub fn from_frame(frame: &Vec<u8>) -> Option<Self> {
        // аутентификация
        let header_size: [u8; 4] = frame[0..4].try_into().unwrap_or_default();
        let cs = crc32(&header_size.to_vec());
        let checksum: [u8; 4] = frame[4..8].try_into().unwrap_or_default();
        if u32::from_be_bytes(checksum) != cs {
            // контрольная сумма не верна
            None
        } else {
            // контрольная сумма верна
            let json_bytes = frame[8..].to_vec();
            let mut json = String::from_utf8(json_bytes).unwrap_or("".to_string());
            json.truncate(u32::from_be_bytes(header_size) as usize);
            match serde_json::from_str::<Self>(json.as_str()) {
                Ok(response) => {
                    let attributes = response.attributes.as_bytes().to_vec();
                    let signature_r = slice2vec(response.signature_r);
                    let signature_s = slice2vec(response.signature_s);
                    let cs = crc32(&[attributes, signature_r, signature_s].concat());
                    if cs != response.checksum {
                        println!("Контрольная сумма ApiResponse НЕ верна!");
                        None
                    } else {
                        Some(response)
                    }
                }
                _ => None,
            }
        }
    }

    pub fn verify(&self, bytes: &Vec<u8>, verify_key: &Point) -> bool {
        let signature = Signature {
            r: vec2bigint(slice2vec(self.signature_r)),
            s: vec2bigint(slice2vec(self.signature_s)),
        };
        let hash = unsafe { digest(bytes) };
        unsafe { verify(&hash, &signature.to_be_bytes(), &verify_key.to_be_bytes()) }
    }
}

pub trait WelsibDtoInterface
where
    Self: Serialize,
{
    fn to_json<T>(&self) -> String
    where
        T: ?Sized + Serialize,
    {
        match serde_json::to_string(self) {
            Ok(json) => json,
            Err(_e) => String::new(),
        }
    }

    fn to_frame<T>(&self) -> Vec<u8>
    where
        T: ?Sized + Serialize,
        Self: Serialize,
    {
        let json = self.to_json::<T>();
        let cs = crc32(&(json.len() as u32).to_be_bytes().to_vec());
        let mut bytes: Vec<u8> = [(json.len() as u32).to_be_bytes(), cs.to_be_bytes()].concat();
        bytes.append(&mut json.as_bytes().to_vec());
        bytes
    }
}

impl WelsibDtoInterface for ApiResponse {}
impl WelsibDtoInterface for ApiRequest {}