use std::io;

use actix::Message;
use byteorder::{BigEndian, ByteOrder};
use bytes::{BufMut, BytesMut};
use serde_derive::{Deserialize, Serialize};
use serde_json::{from_slice, to_string};
use tokio_util::codec::{Decoder, Encoder};

use crate::messages::{UrlGetterMsg, UrlPasterMsg};

#[derive(Serialize, Deserialize, Debug, Message)]
#[rtype(result = "()")]
#[serde(tag = "cmd", content = "data")]
pub enum RpcRequestC {
    Check(UrlGetterMsg),
    Get(UrlGetterMsg),
    Paste(UrlPasterMsg),
    Ping,
}

#[derive(Serialize, Deserialize, Debug, Message)]
#[rtype(result = "()")]
#[serde(tag = "cmd", content = "data")]
pub enum RpcResponseC {
    Proxy(Vec<String>),
    Ping,
}

/// Codec for Client -> Server transport
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
            Ok(Some(from_slice::<RpcRequestC>(&buf)?))
        } else {
            Ok(None)
        }
    }
}

impl Encoder for ToServerCodec {
    type Item = RpcResponseC;
    type Error = io::Error;

    fn encode(&mut self, msg: RpcResponseC, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg = to_string(&msg).unwrap();
        let msg_ref: &[u8] = msg.as_ref();

        dst.reserve(msg_ref.len() + 2);
        dst.put_u16_be(msg_ref.len() as u16);
        dst.put(msg_ref);

        Ok(())
    }
}
