#[macro_use]
extern crate failure;

mod io;
mod ui;

fn main() {
    io::start().unwrap();
}
