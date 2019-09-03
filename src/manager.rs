use actix::{Actor, Addr, AsyncContext, Context, Handler, Message};
use crossbeam::queue::SegQueue;
// use std::{thread};
use std::time::{Duration};

use crate::worker::{Worker, WorkerUrl};

pub struct Manager {
    sq: SegQueue<String>,
    workers: SegQueue<Addr<Worker>>,
    num_workers: i64,
}

impl Actor for Manager {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        for n in 0..self.num_workers {
            Worker::new(n, ctx.address()).start();
        };

        ctx.run_interval(Duration::from_millis(110), move |act, _ctx| {
            if act.sq.len() > 0 && act.workers.len() > 0  {
                let url = act.sq.pop().unwrap();
                let worker = act.workers.pop().unwrap();
                worker.do_send(WorkerUrl{url});
            }
        });
    }
}

impl Manager {
    pub fn new(num: i64) -> Self {
        let sq = SegQueue::new();
        let workers = SegQueue::new();
        Manager { sq, workers, num_workers: num }
    }
}

impl Handler<ManagerUrl> for Manager {
    type Result = ();

    fn handle(&mut self, msg: ManagerUrl, _: &mut Context<Self>) {
        println!("push {} in SegQueue", msg.url);
        self.sq.push(msg.url);
    }
}

impl Handler<WorkRequest> for Manager {
    type Result = ();

    fn handle(&mut self, msg: WorkRequest, _: &mut Context<Self>) {
        println!("worker {} is free, sq len = {}", msg.id, self.sq.len());
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

pub struct ManagerUrl {
    pub url: String,
}

impl Message for ManagerUrl {
    type Result = ();
}

pub struct WorkRequest {
    pub id: i64,
    pub worker: Addr<Worker>,
}

impl Message for WorkRequest {
    type Result = ();
}