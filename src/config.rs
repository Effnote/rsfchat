use std::path::Path;
use std::old_io::File;

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

#[derive(RustcDecodable, Copy)]
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
    let contents = File::open(&Path::new(path)).read_to_string().unwrap();
    let top_level: TopLevel = toml::decode_str(contents.as_slice()).unwrap();
    let TopLevel { user_info, settings } = top_level;
    Config {
        user_info: user_info,
        settings: settings.unwrap_or(DEFAULT_SETTINGS),
    }
}
