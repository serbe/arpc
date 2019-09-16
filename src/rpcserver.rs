use actix::{Actor, Addr, Context, Handler};
use rand::{self, Rng};
use std::collections::HashMap;

use crate::messages::{Connect, Disconnect};
use crate::session::Session;

pub struct RpcServer {
    sessions: HashMap<usize, Addr<Session>>,
}

impl Default for RpcServer {
    fn default() -> RpcServer {
        RpcServer {
            sessions: HashMap::new(),
        }
    }
}

impl Actor for RpcServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for RpcServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        let id = rand::thread_rng().gen::<usize>();
        self.sessions.insert(id, msg.addr);
        id
    }
}

impl Handler<Disconnect> for RpcServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        self.sessions.remove(&msg.id);
    }
}
