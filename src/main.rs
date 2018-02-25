extern crate cursive;
extern crate failure;
extern crate fchat;
extern crate futures;
extern crate tokio_core;
extern crate tokio_timer;

mod ui;
mod io;

fn main() {
    io::start().unwrap();
}
