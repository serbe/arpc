use actix::{Actor, Context, Handler};
use dotenv::var;
use postgres::Connection;
use r2d2::Pool;
use r2d2_postgres::{PostgresConnectionManager, TlsMode};

use crate::messages::ProxyMsg;
use crate::proxy::Proxy;

pub struct DBSaver {
    pub db: Pool<PostgresConnectionManager>,
}

impl DBSaver {
    pub fn new(db: Pool<PostgresConnectionManager>) -> Self {
        DBSaver { db }
    }
}

impl Actor for DBSaver {
    type Context = Context<Self>;
}

impl Handler<ProxyMsg> for DBSaver {
    type Result = ();

    fn handle(&mut self, msg: ProxyMsg, _ctx: &mut Context<Self>) {
        match insert_or_update(&self.db.get().unwrap(), msg.proxy.clone()) {
            Ok(_num) => {
                if msg.proxy.work {
                    println!(
                        "{} work={} anon={} response={}",
                        msg.proxy.hostname, msg.proxy.work, msg.proxy.anon, msg.proxy.response
                    )
                }
            }
            Err(err) => println!("error in db {}", err),
        }
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

pub fn get_work(conn: &Connection, num: i64) -> Vec<String> {
    let mut proxies = Vec::new();
    if let Ok(rows) = &conn.query(
        "SELECT
            hostname
        FROM
            proxies
        WHERE
            work = true AND update_at < (NOW() - interval '1 hour') AND random() < 0.01
        LIMIT $1",
        &[&num],
    ) {
        for row in rows {
            let hostname: String = row.get(0);
            proxies.push(hostname);
        }
    }
    proxies
}
