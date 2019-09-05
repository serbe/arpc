use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler};

use crate::db::DBSaver;
use crate::manager::Manager;
use crate::messages::{ProxyMsg, QuitMsg, UrlMsg, WorkerMsg};
use crate::proxy::{check_proxy};

pub struct Worker {
    id: usize,
    manager: Addr<Manager>,
    db: Addr<DBSaver>,
    my_ip: String,
    target: String,
}

impl Worker {
    pub fn new(
        id: usize,
        manager: Addr<Manager>,
        db: Addr<DBSaver>,
        my_ip: String,
        target: String,
    ) -> Self {
        Worker {
            id,
            manager,
            db,
            my_ip,
            target,
        }
    }
}

impl Actor for Worker {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // println!("worker {} started", self.id);
        self.manager.do_send(WorkerMsg {
            id: self.id,
            worker: ctx.address(),
        });
    }
}

impl Handler<UrlMsg> for Worker {
    type Result = ();

    fn handle(&mut self, msg: UrlMsg, ctx: &mut Context<Self>) -> Self::Result {
        // println!("worker {} get {}", self.id, msg.url);
        match check_proxy(&msg.url, &self.target, &self.my_ip) {
            Ok(proxy) => self.db.do_send(ProxyMsg { proxy }),
            Err(err) => println!("error check proxy {}", err),
        } 
        self.manager.do_send(WorkerMsg {
            id: self.id,
            worker: ctx.address(),
        });
    }
}

impl Handler<QuitMsg> for Worker {
    type Result = ();

    fn handle(&mut self, _msg: QuitMsg, ctx: &mut Context<Self>) -> Self::Result {
        // println!("worker {} get Quit", self.id);
        ctx.stop();
    }
}
