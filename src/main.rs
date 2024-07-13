mod error;
mod io;
mod ui;
pub use error::Error;

fn main() {
    io::start().unwrap();
}
