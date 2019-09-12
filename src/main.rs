use actix::{
    Actor, Addr, AsyncContext, Context, Handler, Message, StreamHandler, SyncArbiter, System,
};
// use actix_web::{web, App, HttpServer};
use dotenv::{dotenv, var};
use futures::Stream;
use std::net;
use std::str::FromStr;
use tokio_codec::FramedRead;
use tokio_io::AsyncRead;
use tokio_tcp::{TcpListener, TcpStream};

use crate::db::{get_connection, DbActor};
// use crate::handlers::{check_type, post_url_list};
use crate::manager::Manager;
use crate::messages::WorkersAddr;
// use crate::server::RpcServer;
use crate::utils::my_ip;
use crate::worker::Worker;
// use crate::session::RpcSession;
// use crate::codec::RpcCodec;

use crate::codec::ToServerCodec;
use crate::server::ChatServer;
use crate::session::ChatSession;

mod codec;
mod db;
// mod handlers;
mod manager;
mod messages;
mod proxy;
mod server;
mod session;
mod utils;
mod worker;

pub struct WebData {
    pub db: Addr<DbActor>,
    pub manager: Addr<Manager>,
}

struct Server {
    chat: Addr<ChatServer>,
    manager: Addr<Manager>,
    db: Addr<DbActor>,
}

impl Actor for Server {
    type Context = Context<Self>;
}

#[derive(Message)]
struct TcpConnect(pub TcpStream, pub net::SocketAddr);

/// Handle stream of TcpStream's
impl Handler<TcpConnect> for Server {
    /// this is response for message, which is defined by `ResponseType` trait
    /// in this case we just return unit.
    type Result = ();

    fn handle(&mut self, msg: TcpConnect, _: &mut Context<Self>) {
        // For each incoming connection we create `ChatSession` actor
        // with out chat server address.
        let server = self.chat.clone();
        let manager = self.manager.clone();
        let db = self.db.clone();
        ChatSession::create(move |ctx| {
            let (r, w) = msg.0.split();
            ChatSession::add_stream(FramedRead::new(r, ToServerCodec), ctx);
            ChatSession::new(
                server,
                manager,
                db,
                actix::io::FramedWrite::new(w, ToServerCodec, ctx),
            )
        });
    }
}

fn main() {
    dotenv().ok();
    let my_ip = my_ip().unwrap();
    let target = var("TARGET").expect("TARGET must be set");
    let num_workers = var("WORKERS")
        .expect("WORKERS must be set")
        .parse::<usize>()
        .unwrap();
    let server_host = var("SERVER").expect("SERVER must be set");
    // let web_server_host = var("WEBSERVER").expect("WEBSERVER must be set");
    let sys = System::new("actix");
    let pool = get_connection();
    let db_saver = DbActor::new(pool.clone()).start();
    // let manager = Manager::new(num_workers).start();
    let manager = Manager::new().start();
    let worker_manager = manager.clone();
    let worker_db = db_saver.clone();
    let workers = SyncArbiter::start(num_workers, move || {
        Worker::new(
            worker_manager.clone(),
            worker_db.clone(),
            my_ip.clone(),
            target.clone(),
        )
    });
    manager.do_send(WorkersAddr(workers));

    // let proxies = get_work(&pool.get().unwrap(), 100);
    // println!("{}", proxies.len());
    // for url in proxies {
    //     manager.do_send(UrlMsg(url));
    // }

    let server_db = db_saver.clone();
    let server_manager = manager.clone();

    let server = ChatServer::default().start();
    let addr = net::SocketAddr::from_str(&server_host).unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    Server::create(|ctx| {
        ctx.add_message_stream(listener.incoming().map_err(|_| ()).map(|st| {
            let addr = st.peer_addr().unwrap();
            TcpConnect(st, addr)
        }));
        Server {
            chat: server,
            manager: server_manager,
            db: server_db,
        }
    });

    // let ctrl_c = tokio_signal::ctrl_c().flatten_stream();
    // let handle_shutdown = ctrl_c
    //     .for_each(|()| {
    //         println!("Ctrl-C received, shutting down");
    //         System::current().stop();
    //         Ok(())
    //     })
    //     .map_err(|_| ());

    // actix::spawn(handle_shutdown);

    // HttpServer::new(move || {
    //     App::new()
    //         .data(WebData {
    //             db: data_db.clone(),
    //             manager: data_manager.clone(),
    //         })
    //         .data(web::JsonConfig::default().limit(4096))
    //         .service(web::resource("/post_url_list").route(web::post().to(post_url_list)))
    //         .service(web::resource("/check/{type}/{limit}").route(web::get().to(check_type)))
    // })
    // .bind(&web_server_host)
    // .unwrap()
    // .start();

    let _ = sys.run();
}
