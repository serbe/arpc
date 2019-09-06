use actix::{Addr, Message};

use crate::proxy::Proxy;
use crate::worker::Worker;

pub struct UrlMsg(pub String);

impl Message for UrlMsg {
    type Result = ();
}

pub struct QuitMsg;

impl Message for QuitMsg {
    type Result = ();
}

pub struct Waiting;

impl Message for Waiting {
    type Result = ();
}

pub struct ProxyMsg(pub Proxy);

impl Message for ProxyMsg {
    type Result = ();
}

pub struct WorkersAddr(pub Addr<Worker>);

impl Message for WorkersAddr {
    type Result = ();
}

pub struct UrlGetterMsg {
    pub limit: i64,
    pub anon: Option<bool>,
    pub work: bool,
    pub hours: Option<i64>,
}

impl Message for UrlGetterMsg {
    type Result = Vec<String>;
}
