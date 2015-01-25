#![feature(slicing_syntax)]
#![allow(unstable)]

extern crate url;
extern crate websocket;
extern crate toml;
extern crate "rustc-serialize" as rustc_serialize;
extern crate hyper;

use std::thread::Thread;
use std::time::Duration;
use std::io::timer::sleep;
use std::sync::mpsc::channel;
use std::borrow::ToOwned;

use url::Url;
use websocket::{Client, Sender, Receiver, Message};

mod ui;
mod message;
mod config;
mod ticket;
mod tabs;

type WsSender = std::sync::mpsc::Sender<String>;

fn main() {
    let config = config::read_config("config.toml");
    let ticket = ticket::get_ticket(&config);

    let url = Url::parse("wss://chat.f-list.net:8799").unwrap();
    let request = Client::connect(url).unwrap();
    let response = request.send().unwrap();
    let (mut sender, mut receiver) = response.begin().split();

    let (received_tx, received_rx) = channel();
    Thread::spawn(move|| {
        for msg in receiver.incoming_messages() {
            let msg = msg.unwrap();
            if let Message::Text(text) = msg {
                message::handle(text, &received_tx);
            }
        }
    });

    let (sender_tx, sender_rx) = channel();
    Thread::spawn(move|| {
        for msg in sender_rx.iter() {
            sender.send_message(Message::Text(msg)).unwrap();
        }
    });

    Thread::spawn({
        let sender = sender_tx.clone();
        move|| -> () {
            loop {
                sender.send("PIN".to_owned()).unwrap();
                sleep(Duration::seconds(35));
            }
        }
    });

    ui::start(received_rx, config, ticket, sender_tx);
}
