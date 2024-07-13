use derive_more::{Display, From};
use futures::channel::mpsc::SendError;

#[derive(From, Debug, Display)]
pub enum Error {
    SendError(SendError),
    Fchat(fchat::Error),
    Std(std::io::Error),
}

impl std::error::Error for Error {}
