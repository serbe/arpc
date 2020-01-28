use std::net::SocketAddr;

use actix::{Addr, Message};
use serde_derive::{Deserialize, Serialize};
use tokio::net::{TcpStream};

use crate::proxy::Proxy;
use crate::session::Session;
use crate::worker::Worker;

#[derive(Message)]
#[rtype(result = "()")]
pub struct UrlMsg(pub String);

#[derive(Message)]
#[rtype(result = "()")]
pub struct QuitMsg;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Waiting;

#[derive(Message)]
#[rtype(result = "()")]
pub struct ProxyMsg(pub Proxy);

#[derive(Message)]
#[rtype(result = "()")]
pub struct WorkersAddr(pub Addr<Worker>);

#[derive(Serialize, Deserialize, Debug)]
pub struct UrlGetterMsg {
    pub limit: i64,
    pub anon: Option<bool>,
    pub work: bool,
    pub hours: Option<i64>,
}

impl Message for UrlGetterMsg {
    type Result = Vec<String>;
}

pub struct Connect {
    pub addr: Addr<Session>,
}

impl Message for Connect {
    type Result = usize;
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct TcpConnect(pub TcpStream, pub SocketAddr);

#[derive(Serialize, Deserialize, Debug)]
pub struct UrlPasterMsg {
    pub urls: Vec<String>,
}
