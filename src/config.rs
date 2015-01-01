use std::path::Path;
use std::io::File;

use super::toml;

#[deriving(RustcDecodable)]
struct TopLevel {
    user_info: Config,
}

#[deriving(RustcDecodable)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub character: String,
}

pub fn read_config(path: &str) -> Config {
    let contents = File::open(&Path::new(path)).read_to_string().unwrap();
    let top_level: TopLevel = toml::decode_str(contents.as_slice()).unwrap();
    top_level.user_info
}
