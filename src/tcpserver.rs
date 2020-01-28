use actix::io::FramedWrite;
use actix::{Actor, Addr, Context, Handler, StreamHandler};
use tokio_util::codec::FramedRead;
// use tokio_io::AsyncRead;

use crate::codec::ToServerCodec;
use crate::manager::Manager;
use crate::messages::TcpConnect;
use crate::pgdb::PgDb;
use crate::rpcserver::RpcServer;
use crate::session::Session;

pub struct TcpServer {
    pub rpc_server: Addr<RpcServer>,
    pub manager: Addr<Manager>,
    pub pg_db: Addr<PgDb>,
}

impl Actor for TcpServer {
    type Context = Context<Self>;
}

impl Handler<TcpConnect> for TcpServer {
    type Result = ();

    fn handle(&mut self, msg: TcpConnect, _: &mut Context<Self>) {
        let rpc_server = self.rpc_server.clone();
        let manager = self.manager.clone();
        let pg_db = self.pg_db.clone();
        Session::create(move |ctx| {
            let (r, w) = msg.0.split();
            Session::add_stream(FramedRead::new(r, ToServerCodec), ctx);
            Session::new(
                rpc_server,
                manager,
                pg_db,
                FramedWrite::new(w, ToServerCodec, ctx),
            )
        });
    }
}
