use actix::{Addr, Message};

use crate::proxy::Proxy;
use crate::worker::Worker;

pub struct UrlMsg {
    pub url: String,
}

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

pub struct ProxyMsg {
    pub proxy: Proxy,
}

impl Message for ProxyMsg {
    type Result = ();
}

pub struct WorkersAddr {
    pub addr: Addr<Worker>,
}

impl Message for WorkersAddr {
    type Result = ();
}
