use actix::{Actor, Addr, Context, Handler, Message};
use actix::AsyncContext;

use crate::manager::{Manager, WorkRequest};

pub struct Worker {
    id: i64,
    manager: Addr<Manager>,
}

impl Worker {
    pub fn new(id: i64, manager: Addr<Manager>) -> Self {
        Worker { id, manager }
    }
}

impl Actor for Worker {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("worker {} started", self.id);
        self.manager.do_send(WorkRequest{ id: self.id, worker: ctx.address() });
    }
}

impl Handler<WorkerUrl> for Worker {
    type Result = ();

    fn handle(&mut self, msg: WorkerUrl, ctx: &mut Context<Self>) -> Self::Result {
        println!("{} get {}", self.id, msg.url);
        self.manager.do_send(WorkRequest{ id: self.id, worker: ctx.address() });
    }
}

pub struct WorkerUrl {
    pub url: String,
}

impl Message for WorkerUrl {
    type Result = ();
}
