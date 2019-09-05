use actix::{Actor, ActorContext, Addr, Handler, SyncContext};

use crate::db::DBSaver;
use crate::manager::Manager;
use crate::messages::{ProxyMsg, QuitMsg, UrlMsg, Waiting};
use crate::proxy::check_proxy;

pub struct Worker {
    manager: Addr<Manager>,
    db: Addr<DBSaver>,
    my_ip: String,
    target: String,
}

impl Worker {
    pub fn new(manager: Addr<Manager>, db: Addr<DBSaver>, my_ip: String, target: String) -> Self {
        Worker {
            manager,
            db,
            my_ip,
            target,
        }
    }
}

impl Actor for Worker {
    type Context = SyncContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        self.manager.do_send(Waiting {});
    }
}

impl Handler<UrlMsg> for Worker {
    type Result = ();

    fn handle(&mut self, msg: UrlMsg, _ctx: &mut SyncContext<Self>) -> Self::Result {
        match check_proxy(&msg.url, &self.target, &self.my_ip) {
            Ok(proxy) => self.db.do_send(ProxyMsg { proxy }),
            Err(err) => println!("error check proxy {}", err),
        }
        self.manager.do_send(Waiting {});
    }
}

impl Handler<QuitMsg> for Worker {
    type Result = ();

    fn handle(&mut self, _msg: QuitMsg, ctx: &mut SyncContext<Self>) -> Self::Result {
        ctx.stop();
    }
}
