use actix::{Actor, ActorContext, Addr, Handler, SyncContext};

use crate::manager::Manager;
use crate::messages::{ProxyMsg, QuitMsg, UrlMsg, Waiting};
use crate::pgdb::PgDb;
use crate::proxy::{check_proxy, Proxy};

pub struct Worker {
    manager: Addr<Manager>,
    pg_db: Addr<PgDb>,
    my_ip: String,
    target: String,
}

impl Worker {
    pub fn new(manager: Addr<Manager>, pg_db: Addr<PgDb>, my_ip: String, target: String) -> Self {
        Worker {
            manager,
            pg_db,
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
        if let Ok(proxy) = Proxy::from(&msg.0) {
            match check_proxy(proxy.clone(), &self.target, &self.my_ip) {
                Ok(checked_proxy) => self.pg_db.do_send(ProxyMsg(checked_proxy)),
                Err(_err) => self.pg_db.do_send(ProxyMsg(proxy)),
            }
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
