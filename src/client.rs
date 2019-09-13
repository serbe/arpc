use std::str::FromStr;
use std::time::Duration;
use std::{io, net, process, thread};

use actix::prelude::*;
use futures::Future;
use tokio_codec::FramedRead;
use tokio_io::io::WriteHalf;
use tokio_io::AsyncRead;
use tokio_tcp::TcpStream;

fn main() -> std::io::Result<()> {
    println!("Running chat client");

    actix::System::run(|| {
        // Connect to server
        let addr = net::SocketAddr::from_str("127.0.0.1:17017").unwrap();
        Arbiter::spawn(
            TcpStream::connect(&addr)
                .and_then(|stream| {
                    let addr = ChatClient::create(|ctx| {
                        let (r, w) = stream.split();
                        ctx.add_stream(FramedRead::new(r, codec::ClientChatCodec));
                        ChatClient {
                            framed: actix::io::FramedWrite::new(w, codec::ClientChatCodec, ctx),
                        }
                    });

                    // start console loop
                    thread::spawn(move || loop {
                        let mut cmd = String::new();
                        if io::stdin().read_line(&mut cmd).is_err() {
                            println!("error");
                            return;
                        }

                        addr.do_send(ClientCommand(cmd));
                    });

                    futures::future::ok(())
                })
                .map_err(|e| {
                    println!("Can not connect to server: {}", e);
                    process::exit(1)
                }),
        );
    })
}

struct ChatClient {
    framed: actix::io::FramedWrite<WriteHalf<TcpStream>, codec::ClientChatCodec>,
}

#[derive(Message)]
struct ClientCommand(String);

impl Actor for ChatClient {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        // start heartbeats otherwise server will disconnect after 10 seconds
        self.hb(ctx)
    }

    fn stopping(&mut self, _: &mut Context<Self>) -> Running {
        println!("Disconnected");

        // Stop application on disconnect
        System::current().stop();

        Running::Stop
    }
}

impl ChatClient {
    fn hb(&self, ctx: &mut Context<Self>) {
        ctx.run_later(Duration::new(1, 0), |act, ctx| {
            act.framed.write(codec::RpcRequestC::Ping);
            act.hb(ctx);
        });
    }
}

impl actix::io::WriteHandler<io::Error> for ChatClient {}

/// Handle stdin commands
impl Handler<ClientCommand> for ChatClient {
    type Result = ();

    fn handle(&mut self, msg: ClientCommand, _: &mut Context<Self>) {
        let m = msg.0.trim();

        // we check for /sss type of messages
        if m.starts_with('/') {
            let v: Vec<&str> = m.splitn(2, ' ').collect();
            match v[0] {
                "/get" => {
                        self.framed.write(codec::RpcRequestC::Get(codec::UrlGetterMsg{limit: 10, anon: None, work: true, hours: None}));
                    }
                "/check" => {
                        self.framed.write(codec::RpcRequestC::Check(codec::UrlGetterMsg{limit: 10, anon: None, work: true, hours: None}));
                    }
                _ => println!("!!! unknown command"),
            }
        }
    }
}

/// Server communication
impl StreamHandler<codec::RpcResponseC, io::Error> for ChatClient {
    fn handle(&mut self, msg: codec::RpcResponseC, _: &mut Context<Self>) {
        match msg {
            codec::RpcResponseC::Proxy(ref msg) => {
                println!("message: {:?}", msg);
            }
            _ => (),
        }
    }
}

#![allow(dead_code)]
use actix::Message;
use byteorder::{BigEndian, ByteOrder};
use bytes::{BufMut, BytesMut};
use serde::{Deserialize, Serialize};
use serde_json as json;
use std::io;
use tokio_io::codec::{Decoder, Encoder};

#[derive(Serialize, Deserialize, Debug)]
pub struct UrlGetterMsg {
    pub limit: i64,
    pub anon: Option<bool>,
    pub work: bool,
    pub hours: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Message)]
#[serde(tag = "cmd", content = "data")]
pub enum RpcRequestC {
    Check(UrlGetterMsg),
    Get(UrlGetterMsg),
    Ping,
}

/// Server response
#[derive(Serialize, Deserialize, Debug, Message)]
#[serde(tag = "cmd", content = "data")]
pub enum RpcResponseC {
    Proxy(Vec<String>),
    Ping,
}

/// Codec for Client -> Server transport
pub struct ChatCodec;

impl Decoder for ChatCodec {
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

impl Encoder for ChatCodec {
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

/// Codec for Server -> Client transport
pub struct ClientChatCodec;

impl Decoder for ClientChatCodec {
    type Item = RpcResponseC;
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
            Ok(Some(json::from_slice::<RpcResponseC>(&buf)?))
        } else {
            Ok(None)
        }
    }
}

impl Encoder for ClientChatCodec {
    type Item = RpcRequestC;
    type Error = io::Error;

    fn encode(&mut self, msg: RpcRequestC, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg = json::to_string(&msg).unwrap();
        let msg_ref: &[u8] = msg.as_ref();

        dst.reserve(msg_ref.len() + 2);
        dst.put_u16_be(msg_ref.len() as u16);
        dst.put(msg_ref);

        Ok(())
    }
}
