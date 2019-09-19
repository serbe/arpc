use std::time::Duration;

use actix::{Actor, Addr, AsyncContext, Context};

use crate::manager::Manager;
use crate::messages::UrlMsg;
use crate::utils::urls_from_dir;

pub struct FWatcher {
    manager: Addr<Manager>,
}

impl FWatcher {
    pub fn new(manager: Addr<Manager>) -> Self {
        FWatcher { manager }
    }
}

impl Actor for FWatcher {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_secs(42), move |act, _ctx| {
            if let Ok(urls) = urls_from_dir() {
                for url in urls {
                    act.manager.do_send(UrlMsg(url));
                }
            }
        });
    }
}
