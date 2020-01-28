use actix::{Actor, Addr, Context, Handler, MessageResult};
use dotenv::var;
use log::{info, warn};
use tokio_postgres::types::ToSql;
use tokio_postgres::{connect, Client, NoTls, Statement};
// use postgres::Connection;
// use r2d2::Pool;
// use r2d2_postgres::{PostgresConnectionManager, TlsMode};

use std::io;

use crate::messages::{ProxyMsg, UrlGetterMsg};
use crate::proxy::Proxy;

pub struct PgConnection  {
    cl: Client,
    // pub db: Pool<PostgresConnectionManager>,
}

impl Actor for PgConnection  {
    type Context = Context<Self>;
}

impl PgConnection  {
    pub async fn connect(db_url: &str) -> Result<Addr<PgConnection>, io::Error> {
        let (cl, _conn) = connect(db_url, NoTls)
            .await
            .expect("can not connect to postgresql");
        // actix_rt::spawn(conn.map(|_| ()));
        Ok(PgConnection::create(move |_| PgConnection {
            cl,
        }))
    }
}

impl Handler<ProxyMsg> for PgConnection {
    type Result = ();

    fn handle(&mut self, msg: ProxyMsg, _ctx: &mut Context<Self>) {
        match insert_or_update(&self.cl, msg.0.clone()) {
            Ok(_num) => {
                if msg.0.work {
                    info!(
                        "{} work={} anon={} response={}",
                        msg.0.hostname, msg.0.work, msg.0.anon, msg.0.response
                    )
                }
            }
            Err(err) => warn!("error in db {}", err),
        }
    }
}

// impl Handler<UrlGetterMsg> for PgConnection {
//     type Result = MessageResult<UrlGetterMsg>;

//     fn handle(&mut self, msg: UrlGetterMsg, _ctx: &mut Context<Self>) -> Self::Result {
//         MessageResult(get_list(&self.db.get().unwrap(), msg))
//     }
// }

// pub fn get_connection() -> Pool<PostgresConnectionManager> {
//     let params = var("PG").expect("PG must be set");
//     let manager = PostgresConnectionManager::new(params.clone(), TlsMode::None)
//         .unwrap_or_else(|_| panic!("Error connection manager to {}", params));
//     Pool::new(manager).expect("error create r2d2 pool")
// }

pub fn insert_or_update(cl: &Client, proxy: Proxy) -> Result<u64, tokio_postgres::Error> {
    let next = &proxy.checks + 1;
    cl.execute(
        "INSERT INTO
            proxies (
                hostname,
                scheme,
                host,
                port,
                work,
                anon,
                response,
                checks,
                create_at,
                update_at
            )
        VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT
            (hostname)
        DO UPDATE SET
            (work, anon, response, checks, update_at) =
            ($5, $6, $7, $11, $10)
        ",
        &[
            &proxy.hostname,
            &proxy.scheme,
            &proxy.host,
            &proxy.port,
            &proxy.work,
            &proxy.anon,
            &proxy.response,
            &proxy.checks,
            &proxy.create_at,
            &proxy.update_at,
            &next,
        ],
    )
}

pub fn get_list(cl: &Client, msg: UrlGetterMsg) -> Vec<String> {
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
    if let Ok(rows) = &cl.query(&query, &[&msg.work, &msg.limit]) {
        for row in rows {
            let hostname: String = row.get(0);
            proxies.push(hostname);
        }
    }
    proxies
}

pub fn get_work(cl: &Client, num: i64) -> Vec<String> {
    let mut proxies = Vec::new();
    if let Ok(rows) = &cl.query(
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
