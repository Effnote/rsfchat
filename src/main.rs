#![feature(slicing_syntax, macro_rules)]

extern crate url;
extern crate websocket;
extern crate toml;
extern crate "rustc-serialize" as rustc_serialize;
extern crate hyper;

use std::thread::Thread;
use std::time::Duration;
use std::io::timer::sleep;
use std::sync::mpsc::channel;

use url::Url;
use websocket::{WebSocketRequest, WebSocketMessage};

mod ui;
mod message;
mod config;
mod ticket;

fn main() {
    let config = config::read_config("config.toml");
    let ticket = ticket::get_ticket(&config);

    let url = Url::parse("wss://chat.f-list.net:8799").unwrap();
    let request = WebSocketRequest::connect(url).unwrap();
    let response = request.send().unwrap();
    let client = response.begin();

    let (received_tx, received_rx) = channel();
    {
        let mut client = client.clone();
        Thread::spawn(move|| {
            for msg in client.incoming_messages() {
                match msg.unwrap() {
                    WebSocketMessage::Text(text) => message::handle(text, &received_tx),
                    _ => {}
                }
            }
        }).detach()
    }

    {
        let mut client = client.clone();
        Thread::spawn(move|| -> () {
            loop {
                client.send_message(WebSocketMessage::Text(String::from_str("PIN")));
                sleep(Duration::seconds(35));
            }
        }).detach()
    }

    ui::start(received_rx, config, ticket, client);
}
