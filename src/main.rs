use actix::{Actor, SyncArbiter, System};
use actix_web::{web, App, HttpServer};
use dotenv::{dotenv, var};

use crate::db::{get_connection, get_work, DBSaver};
use crate::handlers::{check_type, post_url_list};
use crate::manager::Manager;
use crate::messages::{UrlMsg, WorkersAddr};
use crate::utils::my_ip;
use crate::worker::Worker;

mod db;
mod handlers;
mod manager;
mod messages;
mod proxy;
mod utils;
mod worker;

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
    let db_saver = DBSaver::new(pool.clone()).start();
    let manager = Manager::new(num_workers).start();
    let manager_addr = manager.clone();
    let workers = SyncArbiter::start(num_workers, move || {
        Worker::new(
            manager_addr.clone(),
            db_saver.clone(),
            my_ip.clone(),
            target.clone(),
        )
    });
    manager.do_send(WorkersAddr { addr: workers });

    let proxies = get_work(&pool.get().unwrap(), 100);
    println!("{}", proxies.len());
    for url in proxies {
        manager.do_send(UrlMsg { url });
    }

    HttpServer::new(move || {
        App::new()
            .data(manager.clone())
            .data(web::JsonConfig::default().limit(4096))
            .service(web::resource("/post_url_list").route(web::post().to(post_url_list)))
            .service(web::resource("/check/{type}").route(web::get().to(check_type)))
    })
    .bind(&server)
    .unwrap()
    .start();

    let _ = sys.run();
}
