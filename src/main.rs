use db::{get_connection, get_work};
use manager::{Manager, ManagerUrl};
use actix::Actor;

mod db;
mod manager;
mod proxy;
mod utils;
mod worker;

fn main() {
    let sys = actix::System::new("actix");

    let conn = get_connection();
    let manager = Manager::new(10).start();
    let proxies = get_work(&conn, 1000);

    for proxy in proxies {
        let _ = manager.send(ManagerUrl{ url: proxy });
    }

    let _ = sys.run();
}
