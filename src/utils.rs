use std::fs::{create_dir, read_dir, remove_file, File};
use std::io::{BufRead, BufReader, Error, ErrorKind, Result as IoResult};
use std::path::Path;

use regex::Regex;
use rp_client::{client::Client, error::Error as RpError};

pub fn my_ip() -> Result<String, RpError> {
    let mut client = Client::new("https://api.ipify.org").build()?;
    client.send()?;
    client.text()
}

pub fn urls_from_dir() -> IoResult<Vec<String>> {
    let path = Path::new("./watch/");
    let dir = read_dir(path)?;
    let mut urls = Vec::new();
    for entry in dir {
        let entry = entry?;
        let path = entry.path();
        let file = File::open(path.clone())?;
        let f = BufReader::new(file);
        let mut body = String::new();
        for line in f.lines() {
            body.push_str(&line?);
        }
        for line in parge_origins(body)? {
            urls.push(line);
        }
        remove_file(path)?;
    }
    Ok(urls)
}

fn parge_origins(line: String) -> IoResult<Vec<String>> {
    let re = Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}:\d{1,5})")
        .map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))?;
    let mut origins = Vec::new();
    for caps in re.captures_iter(&line) {
        let addr = caps
            .get(1)
            .ok_or_else(|| Error::new(ErrorKind::InvalidData, "error ip:port"))?
            .as_str();
        origins.push(format!("http://{}", addr));
        origins.push(format!("https://{}", addr));
        origins.push(format!("socks5://{}", addr));
    }
    Ok(origins)
}

pub fn create_dir_watch() {
    let _ = create_dir("./watch/");
}
