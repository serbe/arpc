use std::time::Duration;

use actix::{Actor, Addr, AsyncContext, Context, Handler, System};
use crossbeam_queue::SegQueue;
// use dotenv::var;
// use sled::Db;

use crate::messages::{UrlMsg, Waiting, WorkersAddr};
use crate::worker::Worker;

pub struct Manager {
    sq: SegQueue<String>,
    // sled_db: Db,
    workers: Option<Addr<Worker>>,
    free_workers: usize,
}

impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}

impl Manager {
    pub fn new() -> Self {
        let sq = SegQueue::new();
        // let sled_db_name = var("SLED").expect("SLED must be set");
        // let sled_db = Db::open(sled_db_name).unwrap();
        Manager {
            sq,
            // sled_db,
            workers: None,
            free_workers: 0,
        }
    }
}

impl Actor for Manager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_millis(51), move |act, _ctx| {
            if !act.sq.is_empty() && act.free_workers > 0 {
                let url = act.sq.pop().unwrap();
                // if act.sled_db.insert(url.clone(), b"") == Ok(None) {
                if let Some(workers) = &act.workers {
                    act.free_workers -= 1;
                    workers.do_send(UrlMsg(url));
                }
                // }
            }
        });
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        System::current().stop();
    }
}

impl Handler<UrlMsg> for Manager {
    type Result = ();

    fn handle(&mut self, msg: UrlMsg, _: &mut Context<Self>) {
        self.sq.push(msg.0);
    }
}

impl Handler<Waiting> for Manager {
    type Result = ();

    fn handle(&mut self, _msg: Waiting, _: &mut Context<Self>) {
        self.free_workers += 1;
    }
}

impl Handler<WorkersAddr> for Manager {
    type Result = ();

    fn handle(&mut self, msg: WorkersAddr, _: &mut Context<Self>) {
        self.workers = Some(msg.0);
    }
}
