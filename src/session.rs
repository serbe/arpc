use actix::fut::ok;
use actix::io::{FramedWrite, WriteHandler};
use actix::{
    Actor, ActorContext, ActorFuture, Addr, AsyncContext, Context, ContextFutureSpawner, Running,
    StreamHandler, WrapFuture,
};
use std::io;
use std::time::{Duration, Instant};
use tokio_io::io::WriteHalf;
use tokio_tcp::TcpStream;

use crate::codec::{RpcRequestC, RpcResponseC, ToServerCodec};
use crate::db::DbActor;
use crate::manager::Manager;
use crate::messages::{Connect, Disconnect, UrlMsg};
use crate::server::ChatServer;

/// `ChatSession` actor is responsible for tcp peer communications.
pub struct ChatSession {
    /// unique session id
    id: usize,
    /// this is address of chat server
    server: Addr<ChatServer>,
    manager: Addr<Manager>,
    db: Addr<DbActor>,
    /// Client must send ping at least once per 10 seconds, otherwise we drop
    /// connection.
    hb: Instant,
    /// Framed wrapper
    framed: FramedWrite<WriteHalf<TcpStream>, ToServerCodec>,
}

impl Actor for ChatSession {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);

        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        self.server
            .send(Connect {
                addr: ctx.address(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                ok(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify chat server
        self.server.do_send(Disconnect { id: self.id });
        Running::Stop
    }
}

impl WriteHandler<io::Error> for ChatSession {}

/// To use `Framed` with an actor, we have to implement `StreamHandler` trait
impl StreamHandler<RpcRequestC, io::Error> for ChatSession {
    /// This is main event loop for client requests
    fn handle(&mut self, msg: RpcRequestC, ctx: &mut Self::Context) {
        match msg {
            RpcRequestC::Get(url_getter) => {
                self.db
                    .send(url_getter)
                    .into_actor(self) // <- create actor compatible future
                    .then(|res, act, _| {
                        match res {
                            Ok(url_list) => act.framed.write(RpcResponseC::Proxy(url_list)),
                            _ => println!("Something is wrong"),
                        }
                        ok(())
                    })
                    .wait(ctx)
                // .wait(ctx) pauses all events in context,
                // so actor wont receive any new messages until it get list of rooms back
            }
            RpcRequestC::Check(url_getter) => {
                self.db
                    .send(url_getter)
                    .into_actor(self) // <- create actor compatible future
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
                    .wait(ctx)
            }
            // we update heartbeat time on ping from peer
            RpcRequestC::Ping => self.hb = Instant::now(),
        }
    }
}

/// Handler for Message, chat server sends this message, we just send string to
/// peer
// impl Handler<ToSessionMessage> for ChatSession {
//     type Result = ();

//     fn handle(&mut self, msg: ToSessionMessage, _: &mut Self::Context) {
//         // send message to peer
//         self.framed.write(RpcResponseC::Message(msg.0));
//     }
// }

/// Helper methods
impl ChatSession {
    pub fn new(
        server: Addr<ChatServer>,
        manager: Addr<Manager>,
        db: Addr<DbActor>,
        framed: FramedWrite<WriteHalf<TcpStream>, ToServerCodec>,
    ) -> ChatSession {
        ChatSession {
            server,
            manager,
            db,
            framed,
            id: 0,
            hb: Instant::now(),
        }
    }

    /// helper method that sends ping to client every second.
    ///
    /// also this method check heartbeats from client
    fn hb(&self, ctx: &mut Context<Self>) {
        ctx.run_later(Duration::new(1, 0), |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > Duration::new(10, 0) {
                // heartbeat timed out
                println!("Client heartbeat failed, disconnecting!");

                // notify chat server
                act.server.do_send(Disconnect { id: act.id });

                // stop actor
                ctx.stop();
            }

            act.framed.write(RpcResponseC::Ping);
            act.hb(ctx);
        });
    }
}
