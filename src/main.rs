use actix::{Actor, Addr, SyncArbiter, System};
use actix_web::{web, App, HttpServer};
use dotenv::{dotenv, var};

use crate::db::{get_connection, DbActor};
use crate::handlers::{check_type, post_url_list};
use crate::manager::Manager;
use crate::messages::WorkersAddr;
use crate::utils::my_ip;
use crate::worker::Worker;

mod db;
mod handlers;
mod manager;
mod messages;
mod proxy;
mod utils;
mod worker;

pub struct WebData {
    pub db: Addr<DbActor>,
    pub manager: Addr<Manager>,
}

fn main() {
    dotenv().ok();
    let my_ip = my_ip().unwrap();
    let target = var("TARGET").expect("TARGET must be set");
    let num_workers = var("WORKERS")
        .expect("WORKERS must be set")
        .parse::<usize>()
        .unwrap();
    let server = var("SERVER").expect("SERVER must be set");
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

    let data_db = db_saver.clone();
    let data_manager = manager.clone();

    HttpServer::new(move || {
        App::new()
            .data(WebData {
                db: data_db.clone(),
                manager: data_manager.clone(),
            })
            .data(web::JsonConfig::default().limit(4096))
            .service(web::resource("/post_url_list").route(web::post().to(post_url_list)))
            .service(web::resource("/check/{type}/{limit}").route(web::get().to(check_type)))
    })
    .bind(&server)
    .unwrap()
    .start();

    let _ = sys.run();
}
