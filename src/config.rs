use std::path::Path;
use std::fs::File;
use std::io::Read;

use super::toml;

#[derive(RustcDecodable)]
struct TopLevel {
    user_info: UserInfo,
    settings: Option<Settings>,
}

#[derive(RustcDecodable)]
pub struct UserInfo {
    pub username: String,
    pub password: String,
    pub character: String,
}

#[derive(RustcDecodable, Copy, Clone)]
pub struct Settings {
    pub buffer_size: u16,
}

const DEFAULT_SETTINGS: Settings = Settings {
    buffer_size: 400,
};

pub struct Config {
    pub user_info: UserInfo,
    pub settings: Settings,
}

pub fn read_config(path: &str) -> Config {
    let mut contents = String::new();
    File::open(path).unwrap().read_to_string(&mut contents).unwrap();
    let top_level: TopLevel = toml::decode_str(&contents).unwrap();
    let TopLevel { user_info, settings } = top_level;
    Config {
        user_info: user_info,
        settings: settings.unwrap_or(DEFAULT_SETTINGS),
    }
}
