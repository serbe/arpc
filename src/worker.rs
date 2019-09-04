use actix::{Actor, Addr, Context, Handler, Message, ActorContext, AsyncContext};
use std::thread;
use std::time::Duration;

use crate::manager::{Manager, WorkRequest};

pub struct Worker {
    id: usize,
    manager: Addr<Manager>,
}

impl Worker {
    pub fn new(id: usize, manager: Addr<Manager>) -> Self {
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
        println!("worker {} get {}", self.id, msg.url);
        thread::sleep(Duration::from_millis(103));
        self.manager.do_send(WorkRequest{ id: self.id, worker: ctx.address() });
    }
}

impl Handler<Quit> for Worker {
    type Result = ();

    fn handle(&mut self, _msg: Quit, ctx: &mut Context<Self>) -> Self::Result {
        println!("worker {} get Quit", self.id);
        ctx.stop();
    }
}

pub struct WorkerUrl {
    pub url: String,
}

impl Message for WorkerUrl {
    type Result = ();
}

pub struct Quit;

impl Message for Quit {
    type Result = ();
}