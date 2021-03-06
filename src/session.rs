// use std::io;
use std::time::{Duration, Instant};

use actix::{
    fut::ok,
    io::{FramedWrite, WriteHandler},
    Actor, ActorContext, ActorFuture, Addr, AsyncContext, Context, ContextFutureSpawner, Running,
    StreamHandler, WrapFuture,
};
use log::info;
use tokio::io::WriteHalf;
use tokio::net::TcpStream;
use anyhow::Error;

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

impl WriteHandler<Error> for Session {}

impl StreamHandler<Result<RpcRequestC, Error>> for Session {
    fn handle(&mut self, msg: Result<RpcRequestC, Error>, ctx: &mut Self::Context) {
        match msg {
            Ok(RpcRequestC::Get(url_getter)) => self
                .pg_db
                .send(url_getter)
                .into_actor(self)
                .then(|res, act, _| {
                    match res {
                        Ok(url_list) => act.framed.write(RpcResponseC::Proxy(url_list)),
                        _ => info!("Something is wrong"),
                    }
                    ok(())
                })
                .wait(ctx),
            Ok(RpcRequestC::Check(url_getter)) => self
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
                        _ => info!("Something is wrong"),
                    }
                    ok(())
                })
                .wait(ctx),
            Ok(RpcRequestC::Paste(url_paster)) => {
                for url in url_paster.urls {
                    self.manager.do_send(UrlMsg(url));
                }
            }
            Ok(RpcRequestC::Ping) => self.hb = Instant::now(),
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
                info!("Client heartbeat failed, disconnecting!");

                act.rpc_server.do_send(Disconnect { id: act.id });

                ctx.stop();
            }

            act.framed.write(RpcResponseC::Ping);
            act.hb(ctx);
        });
    }
}
