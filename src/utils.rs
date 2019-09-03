// #[derive(Clone)]
// pub struct Config {
//     pub db: String,
//     pub sled: String,
//     pub server: String,
//     pub target: String,
//     pub workers: usize,
// }

// pub fn get_config() -> Result<Config, String> {
//     let db = dotenv::var("PG").map_err(|_| "No found variable PG like postgresql://user[:password]@host[:port][/database][?param1=val1[[&param2=val2]...]] in environment".to_string())?;
//     let sled = dotenv::var("SLED")
//         .map_err(|_| "No found variable sled like SLED in environment".to_string())?;
//     let server = dotenv::var("SERVER")
//         .map_err(|_| "No found variable SERVER like 0.0.0.0:8080 in environment".to_string())?;
//     let target = dotenv::var("TARGET").map_err(|_| {
//         "No found variable target like http://targethost:433/path in environment".to_string()
//     })?;
//     let workers = dotenv::var("WORKERS")
//         .map_err(|_| "No found variable workers like 4 in environment".to_string())?
//         .parse::<usize>()
//         .map_err(|_| "wrong variable workers in environment".to_string())?;
//     Ok(Config {
//         db,
//         sled,
//         server,
//         target,
//         workers,
//     })
// }

// pub fn my_ip() -> Result<String, reqwest::Error> {
//     reqwest::get("https://api.ipify.org")?.text()
// }
