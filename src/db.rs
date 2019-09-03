use crate::proxy::Proxy;
use crossbeam::channel::{select, Receiver};
use postgres::{Connection, TlsMode};
use std::thread;

pub struct DBSaver {
    pub db: Connection,
    pub workers: Receiver<Proxy>,
}

impl DBSaver {
    fn new(db: Connection, workers: Receiver<Proxy>) -> Self {
        DBSaver { db, workers }
    }

    pub fn start(db: Connection, workers: Receiver<Proxy>) {
        let db_saver = DBSaver::new(db, workers);
        thread::spawn(move || db_saver.run());
    }

    fn run(&self) {
        loop {
            select! {
                recv(self.workers) -> msg => {
                    if let Ok(proxy) = msg {
                        let _ = insert_or_update(&self.db, proxy);
                    }
                }
            }
        }
    }
}

pub fn get_connection(params: &str) -> Result<Connection, String> {
    Connection::connect(params, TlsMode::None).map_err(|e| e.to_string())
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
