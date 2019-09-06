use actix::{Actor, Context, Handler, MessageResult};
use dotenv::var;
use postgres::Connection;
use r2d2::Pool;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};

use crate::messages::{ProxyMsg, UrlGetterMsg};
use crate::proxy::Proxy;

pub struct DbActor {
    pub db: Pool<PostgresConnectionManager>,
}

impl DbActor {
    pub fn new(db: Pool<PostgresConnectionManager>) -> Self {
        DbActor { db }
    }
}

impl Actor for DbActor {
    type Context = Context<Self>;
}

impl Handler<ProxyMsg> for DbActor {
    type Result = ();

    fn handle(&mut self, msg: ProxyMsg, _ctx: &mut Context<Self>) {
        match insert_or_update(&self.db.get().unwrap(), msg.0.clone()) {
            Ok(_num) => {
                if msg.0.work {
                    println!(
                        "{} work={} anon={} response={}",
                        msg.0.hostname, msg.0.work, msg.0.anon, msg.0.response
                    )
                }
            }
            Err(err) => println!("error in db {}", err),
        }
    }
}

impl Handler<UrlGetterMsg> for DbActor {
    type Result = MessageResult<UrlGetterMsg>;

    fn handle(&mut self, msg: UrlGetterMsg, _ctx: &mut Context<Self>) -> Self::Result {
        MessageResult(get_list(&self.db.get().unwrap(), msg))
    }
}

pub fn get_connection() -> Pool<PostgresConnectionManager> {
    let params = var("PG").expect("PG must be set");
    let manager = PostgresConnectionManager::new(params.clone(), TlsMode::None)
        .unwrap_or_else(|_| panic!("Error connection manager to {}", params));
    Pool::new(manager).expect("error create r2d2 pool")
}

pub fn insert_or_update(conn: &Connection, proxy: Proxy) -> Result<u64, String> {
    conn.execute(
        "INSERT INTO
            proxies (work, anon, checks, hostname, host, port, scheme, create_at, update_at, response)
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT
            (hostname)
        DO UPDATE SET
            (work, anon, checks, update_at, response) =
            ($1, $2, $3 + 1, $9, $10)
        ",
        &[&proxy.work, &proxy.anon, &proxy.checks, &proxy.hostname, &proxy.host, &proxy.port, &proxy.scheme, &proxy.create_at, &proxy.update_at, &proxy.response]).map_err(|e| format!("error insert {}", e.to_string()))
}

pub fn get_list(conn: &Connection, msg: UrlGetterMsg) -> Vec<String> {
    let mut proxies = Vec::new();
    let anon = match msg.anon {
        Some(value) => format!("AND anon = {}", value),
        None => String::new(),
    };
    let hours = match msg.hours {
        Some(value) => format!("AND update_at < (NOW() - interval '{} hour')", value),
        None => String::new(),
    };
    let query = format!(
        "SELECT
            hostname
        FROM
            proxies
        WHERE
            work = $1 {} {} AND random() < 0.01
        LIMIT $2",
        anon, hours
    );
    if let Ok(rows) = &conn.query(&query, &[&msg.work, &msg.limit]) {
        for row in rows {
            let hostname: String = row.get(0);
            proxies.push(hostname);
        }
    }
    proxies
}

// pub fn get_work(conn: &Connection, num: i64) -> Vec<String> {
//     let mut proxies = Vec::new();
//     if let Ok(rows) = &conn.query(
//         "SELECT
//             hostname
//         FROM
//             proxies
//         WHERE
//             work = true AND update_at < (NOW() - interval '1 hour') AND random() < 0.01
//         LIMIT $1",
//         &[&num],
//     ) {
//         for row in rows {
//             let hostname: String = row.get(0);
//             proxies.push(hostname);
//         }
//     }
//     proxies
// }
