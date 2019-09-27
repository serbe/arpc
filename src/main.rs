use std::net;
use std::str::FromStr;

use actix::{Actor, AsyncContext, SyncArbiter, System};
use dotenv::{dotenv, var};
use futures::future::Future;
use futures::Stream;
use log::info;
use tokio_tcp::TcpListener;

use crate::fwatcher::FWatcher;
use crate::manager::Manager;
use crate::messages::{TcpConnect, WorkersAddr};
use crate::pgdb::{get_connection, PgDb};
use crate::rpcserver::RpcServer;
use crate::tcpserver::TcpServer;
use crate::utils::{create_dir_watch, my_ip};
use crate::worker::Worker;

mod codec;
mod fwatcher;
mod manager;
mod messages;
mod pgdb;
mod proxy;
mod rpcserver;
mod session;
mod tcpserver;
mod utils;
mod worker;

fn main() {
    // set_var("RUST_LOG", "info");
    dotenv().ok();
    env_logger::init();
    create_dir_watch();
    info!("app started");
    let my_ip = my_ip().unwrap();
    let target = var("TARGET").expect("TARGET must be set");
    let num_workers = var("WORKERS")
        .expect("WORKERS must be set")
        .parse::<usize>()
        .unwrap();
    let server_host = var("SERVER").expect("SERVER must be set");
    let sys = System::new("actix");
    let pool = get_connection();
    let pg_db = PgDb::new(pool.clone()).start();
    let manager = Manager::new().start();
    let worker_manager = manager.clone();
    let worker_db = pg_db.clone();
    let workers = SyncArbiter::start(num_workers, move || {
        Worker::new(
            worker_manager.clone(),
            worker_db.clone(),
            my_ip.clone(),
            target.clone(),
        )
    });
    manager.do_send(WorkersAddr(workers));

    let _ = FWatcher::new(manager.clone()).start();

    let rpc_server = RpcServer::default().start();
    let addr = net::SocketAddr::from_str(&server_host).unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    TcpServer::create(|ctx| {
        ctx.add_message_stream(listener.incoming().map_err(|_| ()).map(|st| {
            let addr = st.peer_addr().unwrap();
            TcpConnect(st, addr)
        }));
        TcpServer {
            rpc_server,
            manager,
            pg_db,
        }
    });

    let ctrl_c = tokio_signal::ctrl_c().flatten_stream();
    let handle_shutdown = ctrl_c
        .for_each(|()| {
            info!("Ctrl-C received, shutting down");
            System::current().stop();
            Ok(())
        })
        .map_err(|_| ());

    actix::spawn(handle_shutdown);

    let _ = sys.run();
}
