use actix::{Actor, Message, Handler};

pub struct Worker {
    id: i64,
    busy: bool,
}

impl Actor for Worker {
    type Context = Context<Self>;
}

impl Handler<WorkerMsg> for Worker {
    type Result = i64;

    fn handle(&mut self, msg: WorkerMsg, _: &mut Context<Self>) -> Self::Result {
        println!("{} get {}", self.id, msg.url);
        self.id
    }
}

pub struct WorkerMsg {
    url: String
}

impl Message for WorkerMsg {
    type result = String;
}

