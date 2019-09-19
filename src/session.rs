use std::io;
use std::time::{Duration, Instant};

use actix::fut::ok;
use actix::io::{FramedWrite, WriteHandler};
use actix::{
    Actor, ActorContext, ActorFuture, Addr, AsyncContext, Context, ContextFutureSpawner, Running,
    StreamHandler, WrapFuture,
};
use tokio_io::io::WriteHalf;
use tokio_tcp::TcpStream;

use crate::codec::{RpcRequestC, RpcResponseC, ToServerCodec};
use crate::manager::Manager;
use crate::messages::{Connect, Disconnect, UrlMsg};
use crate::pgdb::PgDb;
use crate::rpcserver::RpcServer;

pub struct Session {
    id: usize,
    rpc_server: Addr<RpcServer>,
    manager: Addr<Manager>,
    pg_db: Addr<PgDb>,
    hb: Instant,
    framed: FramedWrite<WriteHalf<TcpStream>, ToServerCodec>,
}

impl Actor for Session {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        self.rpc_server
            .send(Connect {
                addr: ctx.address(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    _ => ctx.stop(),
                }
                ok(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.rpc_server.do_send(Disconnect { id: self.id });
        Running::Stop
    }
}

impl WriteHandler<io::Error> for Session {}

impl StreamHandler<RpcRequestC, io::Error> for Session {
    fn handle(&mut self, msg: RpcRequestC, ctx: &mut Self::Context) {
        match msg {
            RpcRequestC::Get(url_getter) => self
                .pg_db
                .send(url_getter)
                .into_actor(self)
                .then(|res, act, _| {
                    match res {
                        Ok(url_list) => act.framed.write(RpcResponseC::Proxy(url_list)),
                        _ => println!("Something is wrong"),
                    }
                    ok(())
                })
                .wait(ctx),
            RpcRequestC::Check(url_getter) => self
                .pg_db
                .send(url_getter)
                .into_actor(self)
                .then(|res, act, _| {
                    match res {
                        Ok(url_list) => {
                            for url in url_list {
                                act.manager.do_send(UrlMsg(url));
                            }
                        }
                        _ => println!("Something is wrong"),
                    }
                    ok(())
                })
                .wait(ctx),
            RpcRequestC::Ping => self.hb = Instant::now(),
        }
    }
}

impl Session {
    pub fn new(
        rpc_server: Addr<RpcServer>,
        manager: Addr<Manager>,
        pg_db: Addr<PgDb>,
        framed: FramedWrite<WriteHalf<TcpStream>, ToServerCodec>,
    ) -> Session {
        Session {
            rpc_server,
            manager,
            pg_db,
            framed,
            id: 0,
            hb: Instant::now(),
        }
    }

    fn hb(&self, ctx: &mut Context<Self>) {
        ctx.run_later(Duration::new(1, 0), |act, ctx| {
            if Instant::now().duration_since(act.hb) > Duration::new(10, 0) {
                println!("Client heartbeat failed, disconnecting!");

                act.rpc_server.do_send(Disconnect { id: act.id });

                ctx.stop();
            }

            act.framed.write(RpcResponseC::Ping);
            act.hb(ctx);
        });
    }
}
