// use chrono::{DateTime, Local};
// use std::time::Instant;

// #[derive(Debug)]
// pub struct Proxy {
//     pub insert: bool,
//     pub update: bool,
//     pub work: bool,
//     pub anon: bool,
//     pub checks: i32,
//     pub hostname: String,
//     pub host: String,
//     pub port: String,
//     pub scheme: String,
//     pub create_at: DateTime<Local>,
//     pub update_at: DateTime<Local>,
//     pub response: i64,
// }

// impl Proxy {
//     pub fn from(s: &str) -> Result<Self, String> {
//         let raw = s;

//         if raw.contains('#') {
//             Err(format!("hostname contain fragment {}", raw))?
//         }

//         if raw.contains('?') {
//             Err(format!("hostname contain query {}", raw))?
//         }

//         let (raw, scheme) = if let Some(pos) = raw.find("://") {
//             (
//                 raw.get(pos + 3..)
//                     .ok_or_else(|| format!("not parse scheme {}", raw))?,
//                 raw.get(..pos)
//                     .ok_or_else(|| format!("not parse scheme {}", raw))?
//                     .to_string(),
//             )
//         } else {
//             Err(format!("hostname not contain scheme {}", raw))?
//         };

//         if raw.contains('@') {
//             Err(format!("user info in hostname not supported {}", raw))?
//         };

//         if raw.contains('/') {
//             Err(format!("hostname contain path {}", raw))?
//         };

//         let (host, port) = if let Some(pos) = raw.rfind(':') {
//             if let Some(start) = raw.find('[') {
//                 if let Some(end) = raw.find(']') {
//                     if start == 0 && pos == end + 1 {
//                         (
//                             raw.get(..pos)
//                                 .ok_or_else(|| format!("not parse host {}", raw))?
//                                 .to_string(),
//                             raw.get(pos + 1..)
//                                 .ok_or_else(|| format!("not parse port {}", raw))?
//                                 .to_string(),
//                         )
//                     } else {
//                         Err(format!("not parse ipv6 {}", raw))?
//                     }
//                 } else {
//                     Err(format!("not parse ipv6 {}", raw))?
//                 }
//             } else {
//                 (
//                     raw.get(..pos)
//                         .ok_or_else(|| format!("not parse host {}", raw))?
//                         .to_string(),
//                     raw.get(pos + 1..)
//                         .ok_or_else(|| format!("not parse port {}", raw))?
//                         .to_string(),
//                 )
//             }
//         } else {
//             Err(format!("not parse port {}", raw))?
//         };

//         let _ = port
//             .parse::<u32>()
//             .map_err(|_| format!("not parse port {}", port))?;

//         Ok(Proxy {
//             insert: false,
//             update: false,
//             work: false,
//             anon: false,
//             checks: 0,
//             hostname: format!("{}:{}", host, port),
//             host,
//             port,
//             scheme,
//             create_at: chrono::Local::now(),
//             update_at: chrono::Local::now(),
//             response: 0,
//         })
//     }
// }

// pub fn check_proxy(proxy_url: &str, target_url: &str, my_ip: &str) -> Result<Proxy, String> {
//     let dur = Instant::now();
//     let proxy = reqwest::Proxy::all(proxy_url)
//         .map_err(|e| format!("set proxy {} error: {}", proxy_url, e.to_string()))?;
//     let client = reqwest::Client::builder()
//         .proxy(proxy)
//         .build()
//         .map_err(|e| format!("client builder error {}", e.to_string()))?;
//     let body = client
//         .get(target_url)
//         .send()
//         .map_err(|e| format!("get via {} {}", proxy_url, e.to_string()))?
//         .text()
//         .map_err(|e| format!("convert to text error {}", e.to_string()))?;
//     let mut proxy = Proxy::from(proxy_url)?;
//     proxy.work = true;
//     if !body.contains(my_ip) && body.matches("<p>").count() == 1 {
//         proxy.anon = true;
//     }
//     proxy.create_at = Local::now();
//     proxy.update_at = Local::now();
//     proxy.response = dur.elapsed().as_micros() as i64;
//     Ok(proxy)
// }
