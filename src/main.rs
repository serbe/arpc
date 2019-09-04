use db::{get_connection, get_work};
use manager::{Manager, ManagerUrl};
use actix::{Actor, System};

mod db;
mod manager;
mod proxy;
mod utils;
mod worker;

fn main() {
    let sys = System::new("actix");

    let conn = get_connection();
    let manager = Manager::new(10).start();
    let proxies = get_work(&conn, 20);
    println!("{}", proxies.len());
    for proxy in proxies {
        manager.do_send(ManagerUrl{ url: proxy });
    }

    let _ = sys.run();
}
