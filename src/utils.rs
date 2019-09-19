use std::fs::{create_dir, read_dir, remove_file, File};
use std::io::{BufRead, BufReader, Error, ErrorKind, Result as IoResult};
use std::path::Path;

use regex::Regex;

pub fn my_ip() -> Result<String, reqwest::Error> {
    reqwest::get("https://api.ipify.org")?.text()
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
    let re = Regex::new(r"(https|http|socks5).+?(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\:\d{1,5})")
        .map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))?;
    let mut origins = Vec::new();
    for caps in re.captures_iter(&line) {
        origins.push(format!(
            "{}://{}",
            caps.get(0)
                .ok_or_else(|| Error::new(ErrorKind::InvalidData, "error schema"))?
                .as_str(),
            caps.get(1)
                .ok_or_else(|| Error::new(ErrorKind::InvalidData, "error ip port"))?
                .as_str()
        ))
    }
    Ok(origins)
}

pub fn create_dir_watch() {
    let _ = create_dir("./watch/");
}
