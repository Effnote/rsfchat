use super::hyper;
use url::percent_encoding::{utf8_percent_encode_to, FORM_URLENCODED_ENCODE_SET};
use rustc_serialize::json;
use config::Config;

#[derive(RustcDecodable, Show)]
pub struct Ticket {
    pub characters: Vec<String>,
    pub default_character: String,
    pub ticket: String,
}

pub fn get_ticket(config: &Config) -> Ticket {
    let mut client = hyper::Client::new();
    let mut body = "account=".to_string();
    utf8_percent_encode_to(&*config.user_info.username, FORM_URLENCODED_ENCODE_SET, &mut body);
    body.push_str("&password=");
    utf8_percent_encode_to(&*config.user_info.password, FORM_URLENCODED_ENCODE_SET, &mut body);
    let mime = "application/x-www-form-urlencoded".parse().unwrap();
    let mut response = client.post("http://www.f-list.net/json/getApiTicket.php")
        .body(body.as_slice())
        .header(hyper::header::ContentType(mime))
        .send().unwrap();
    let response = response.read_to_string().unwrap();
    let ticket: Ticket = json::decode(response.as_slice()).unwrap(); // TODO: Handle the possibility of an response with an error.
    ticket
}
