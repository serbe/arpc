use actix::{Addr, Message};
use serde::{Deserialize, Serialize};
// use std::net::SocketAddr;
// use tokio_tcp::{TcpStream};

use crate::proxy::Proxy;
use crate::session::ChatSession;
use crate::worker::Worker;

#[derive(Message)]
pub struct UrlMsg(pub String);

#[derive(Message)]
pub struct QuitMsg;

#[derive(Message)]
pub struct Waiting;

#[derive(Message)]
pub struct ProxyMsg(pub Proxy);

#[derive(Message)]
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

/// Chat server sends this messages to session
// #[derive(Message)]
// pub struct ToSessionMessage(pub String);

/// New chat session is created
pub struct Connect {
    pub addr: Addr<ChatSession>,
}

/// Response type for Connect message
///
/// Chat server returns unique session id
impl Message for Connect {
    type Result = usize;
}

/// Session is disconnected
#[derive(Message)]
pub struct Disconnect {
    pub id: usize,
}

// /// Join room, if room does not exists create new one.
// #[derive(Message)]
// pub struct Join {
//     /// Client id
//     pub id: usize,
//     /// Room name
//     pub name: String,
// }
