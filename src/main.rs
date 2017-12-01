extern crate cursive;
extern crate fchat;
extern crate futures;
extern crate tokio_core;
extern crate tokio_timer;

mod ui;
mod config;
mod controller;

fn main() {
    controller::start().unwrap();
}
