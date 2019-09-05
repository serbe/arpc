use actix::{Actor, System};
use dotenv::{dotenv, var};

use crate::db::{get_connection, get_work, DBSaver};
use crate::manager::Manager;
use crate::messages::UrlMsg;
use crate::utils::my_ip;
use crate::worker::Worker;

mod db;
mod manager;
mod messages;
mod proxy;
mod utils;
mod worker;

fn main() {
    dotenv().ok();
    let sys = System::new("actix");
    let pool = get_connection();
    let db_saver = DBSaver::new(pool.clone()).start();
    let num_workers = var("WORKERS").unwrap().parse::<usize>().unwrap();
    let manager = Manager::new(num_workers.clone()).start();
    let my_ip = my_ip().unwrap();
    let target = var("TARGET").unwrap();
    for n in 0..num_workers {
        Worker::new(
            n,
            manager.clone(),
            db_saver.clone(),
            my_ip.clone(),
            target.clone(),
        )
        .start();
    }

    let proxies = get_work(&pool.get().unwrap(), 20);
    // println!("{}", proxies.len());
    for url in proxies {
        manager.do_send(UrlMsg { url });
    }

    let _ = sys.run();
}
