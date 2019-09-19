use std::fs::{create_dir, read_dir, remove_file, File};
use std::io::{BufRead, BufReader, Result as IoResult};
use std::path::Path;

// use regex::Regex;

pub fn my_ip() -> Result<String, reqwest::Error> {
    reqwest::get("https://api.ipify.org")?.text()
}

pub fn urls_from_dir() -> IoResult<Vec<String>> {
    let path = Path::new("./watch/");
    let dir = read_dir(path)?;
    let mut urls = Vec::new();
    // let re = Regex::new(r"(https|http|socks5).*?(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)").unwrap();
    for entry in dir {
        let entry = entry?;
        let path = entry.path();
        let file = File::open(path.clone())?;
        let f = BufReader::new(file);
        for line in f.lines() {
            urls.push(line?);
        }
        remove_file(path)?;
    }
    Ok(urls)
}

pub fn create_dir_watch() {
    let _ = create_dir("./watch/");
}
