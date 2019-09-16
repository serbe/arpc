use actix::Message;
use byteorder::{BigEndian, ByteOrder};
use bytes::{BufMut, BytesMut};
use serde::{Deserialize, Serialize};
use serde_json as json;
use std::io;
use tokio_io::codec::{Decoder, Encoder};

use crate::messages::UrlGetterMsg;

#[derive(Serialize, Deserialize, Debug, Message)]
#[serde(tag = "cmd", content = "data")]
pub enum RpcRequestC {
    Check(UrlGetterMsg),
    Get(UrlGetterMsg),
    Ping,
}

#[derive(Serialize, Deserialize, Debug, Message)]
#[serde(tag = "cmd", content = "data")]
pub enum RpcResponseC {
    Proxy(Vec<String>),
    Ping,
}

pub struct ToServerCodec;

impl Decoder for ToServerCodec {
    type Item = RpcRequestC;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let size = {
            if src.len() < 2 {
                return Ok(None);
            }
            BigEndian::read_u16(src.as_ref()) as usize
        };

        if src.len() >= size + 2 {
            src.split_to(2);
            let buf = src.split_to(size);
            Ok(Some(json::from_slice::<RpcRequestC>(&buf)?))
        } else {
            Ok(None)
        }
    }
}

impl Encoder for ToServerCodec {
    type Item = RpcResponseC;
    type Error = io::Error;

    fn encode(&mut self, msg: RpcResponseC, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg = json::to_string(&msg).unwrap();
        let msg_ref: &[u8] = msg.as_ref();

        dst.reserve(msg_ref.len() + 2);
        dst.put_u16_be(msg_ref.len() as u16);
        dst.put(msg_ref);

        Ok(())
    }
}
