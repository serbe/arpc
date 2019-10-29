use std::time::Instant;

use chrono::{Local, NaiveDateTime};
use rp_client::{client::Client, error::Error as RpError};

#[derive(Clone, Debug)]
pub struct Proxy {
    pub insert: bool,
    pub update: bool,
    pub work: bool,
    pub anon: bool,
    pub checks: i32,
    pub hostname: String,
    pub host: String,
    pub port: i32,
    pub scheme: String,
    pub create_at: NaiveDateTime,
    pub update_at: NaiveDateTime,
    pub response: i64,
}

impl Proxy {
    pub fn from(s: &str) -> Result<Self, String> {
        let raw = s;

        if raw.contains('#') {
            return Err(format!("hostname contain fragment {}", raw));
        }

        if raw.contains('?') {
            return Err(format!("hostname contain query {}", raw));
        }

        let (raw, scheme) = if let Some(pos) = raw.find("://") {
            (
                raw.get(pos + 3..)
                    .ok_or_else(|| format!("not parse scheme {}", raw))?,
                raw.get(..pos)
                    .ok_or_else(|| format!("not parse scheme {}", raw))?
                    .to_string(),
            )
        } else {
            return Err(format!("hostname not contain scheme {}", raw));
        };

        if raw.contains('@') {
            return Err(format!("user info in hostname not supported {}", raw));
        };

        if raw.contains('/') {
            return Err(format!("{} hostname contain path {}", s, raw));
        };

        let (host, s_port) = if let Some(pos) = raw.rfind(':') {
            if let Some(start) = raw.find('[') {
                if let Some(end) = raw.find(']') {
                    if start == 0 && pos == end + 1 {
                        (
                            raw.get(..pos)
                                .ok_or_else(|| format!("not parse host {}", raw))?
                                .to_string(),
                            raw.get(pos + 1..)
                                .ok_or_else(|| format!("not parse port {}", raw))?
                                .to_string(),
                        )
                    } else {
                        return Err(format!("not parse ipv6 {}", raw));
                    }
                } else {
                    return Err(format!("not parse ipv6 {}", raw));
                }
            } else {
                (
                    raw.get(..pos)
                        .ok_or_else(|| format!("not parse host {}", raw))?
                        .to_string(),
                    raw.get(pos + 1..)
                        .ok_or_else(|| format!("not parse port {}", raw))?
                        .to_string(),
                )
            }
        } else {
            return Err(format!("not parse port {}", raw));
        };

        let port = s_port
            .parse::<i32>()
            .map_err(|_| format!("not parse port {}", s_port))?;
        if port < 0 || port > 65535 {
            return Err("wrong port".to_string());
        }

        Ok(Proxy {
            insert: false,
            update: false,
            work: false,
            anon: false,
            checks: 0,
            hostname: format!("{}://{}:{}", scheme, host, s_port),
            host,
            port,
            scheme,
            create_at: Local::now().naive_local(),
            update_at: Local::now().naive_local(),
            response: 0,
        })
    }
}

pub fn check_proxy(proxy: Proxy, target_url: &str, my_ip: &str) -> Result<Proxy, RpError> {
    let dur = Instant::now();
    let mut proxy = proxy;
    let mut client = Client::new(target_url).proxy(&proxy.hostname).build()?;
    let response = client.send()?;
    if !response.status_code().is_success() {
        return Err(RpError::StatusErr);
    }
    let body = client.text()?;
    proxy.work = true;
    if !body.contains(&my_ip) && body.matches("<p>").count() == 1 {
        proxy.anon = true;
    }
    proxy.response = dur.elapsed().as_micros() as i64;
    Ok(proxy)
}
