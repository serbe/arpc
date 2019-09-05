pub fn my_ip() -> Result<String, reqwest::Error> {
    reqwest::get("https://api.ipify.org")?.text()
}
