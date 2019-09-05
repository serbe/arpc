use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, System};
use crossbeam::queue::SegQueue;
// use dotenv::var;
use std::time::Duration;

// use crate::db::DBSaver;
use crate::messages::{QuitMsg, UrlMsg, WorkerMsg};
// use crate::utils::my_ip;
use crate::worker::Worker;

pub struct Manager {
    sq: SegQueue<String>,
    workers: SegQueue<Addr<Worker>>,
    // db: Addr<DBSaver>,
    num_workers: usize,
}

impl Manager {
    pub fn new(num: usize) -> Self {
        let sq = SegQueue::new();
        let workers = SegQueue::new();
        Manager {
            sq,
            workers,
            // db,
            num_workers: num,
        }
    }
}

impl Actor for Manager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_millis(51), move |act, ctx| {
            if act.sq.len() > 0 && act.workers.len() > 0 {
                let url = act.sq.pop().unwrap();
                let worker = act.workers.pop().unwrap();
                // println!("sq len {} workers free {}", act.sq.len(), act.workers.len());
                worker.do_send(UrlMsg { url });
            } else if act.sq.len() == 0 && act.workers.len() == act.num_workers {
                ctx.stop();
            }
        });
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        // println!("stop manager");
        while !self.workers.is_empty() {
            self.workers.pop().unwrap().do_send(QuitMsg {});
        }
        System::current().stop();
    }
}

impl Handler<UrlMsg> for Manager {
    type Result = ();

    fn handle(&mut self, msg: UrlMsg, _: &mut Context<Self>) {
        // println!("push {} in SegQueue", msg.url);
        self.sq.push(msg.url);
    }
}

impl Handler<WorkerMsg> for Manager {
    type Result = ();

    fn handle(&mut self, msg: WorkerMsg, _: &mut Context<Self>) {
        // println!("worker {} is free, sq len = {}", msg.id, self.sq.len());
        self.workers.push(msg.worker);
        // let millis = Duration::from_millis(100);
        // loop {
        //     if let Ok(url) = self.sq.pop() {
        //         msg.worker.do_send(WorkerUrl{ url });
        //         break;
        //     } else {
        //         thread::sleep(millis);
        //     }
        // }
    }
}
