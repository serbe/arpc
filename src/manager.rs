use actix::{Actor, Context, Handler, Message};
use crossbeam::queue::SegQueue;

pub struct Manager {
    sq: SegQueue<String>
}

impl Actor for Manager {
    type Context = Context<Self>;
}


impl Manager {
    pub fn new() -> Self {
        let sq = SegQueue::new();
        Manager {
            sq
        }
    }
}

impl Handler<ManagerMsg> for Manager {
    type Result = ();

    fn handle(&mut self, msg: ManagerMsg, _: &mut Context<Self>) {
        self.sq.push(msg.url);
    }
}

pub struct ManagerMsg {
    url: String
}

impl Message for ManagerMsg {
    type Result = ();
}