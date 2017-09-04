extern crate cursive;
extern crate fchat;
extern crate futures;
extern crate tokio_core;

use fchat::{Server, Ticket};
use fchat::message::{client, server};
use futures::{Future, Stream, Sink};
use futures::sync::mpsc::{channel, Receiver, Sender};

use tokio_core::reactor::Core;

mod ui;

fn connect(ticket: &Ticket, character: String, rx: Receiver<client::Message>, tx: Sender<server::Message>) {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let chat = fchat::connect(Server::Debug, &handle)
        .and_then(|(sink, stream)| {
            (
                fchat::identify(
                    sink,
                    ticket,
                    character,
                    "RSFChat".to_owned(),
                    "0.0.1".to_owned(),
                ),
                Ok(stream),
            )
        })
        .and_then(|(_sink, stream)| {
            stream.forward(tx.sink_map_err(|_| fchat::Error::Channel))
        });
    core.run(chat).unwrap();
}

fn main() {
    ui::start();
}
