use lazy_static::lazy_static;
use serde::Deserialize;
use serde_json;

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub url: String,
    pub ssh_url: String,
}

lazy_static! {
    pub static ref CONFIG: Config = {
        let data = std::fs::read("config.json").unwrap();
        let data = std::str::from_utf8(&data).unwrap();
        serde_json::from_str(data).unwrap()
    };
}
