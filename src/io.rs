use std;
use std::time::Duration;

use fchat::{self, Server};

use tokio_core::reactor::{Core, Handle};
use tokio_timer::Timer;

use futures::{self, Future, Sink, Stream};
use futures::sync::mpsc::{Sender, UnboundedSender};

use failure::Error;

use ui;

pub enum Event {
    Connect {
        server: Server,
        ticket: fchat::Ticket,
        character: String,
    },
    SendMessage(fchat::message::client::Message),
    ReceivedMessage(fchat::message::server::Message),
}

struct NetworkController {
    handle: Handle,
    event_tx: UnboundedSender<Event>,
    ui_sender: std::sync::mpsc::Sender<ui::Event>,
    server: Option<Server>,
    fchat_tx: Option<Sender<fchat::message::client::Message>>,
    character: Option<String>,
}

impl NetworkController {
    fn new(
        handle: Handle,
        ui_sender: std::sync::mpsc::Sender<ui::Event>,
        event_tx: UnboundedSender<Event>,
    ) -> NetworkController {
        NetworkController {
            handle,
            fchat_tx: None,
            event_tx,
            ui_sender,
            server: None,
            character: None,
        }
    }

    fn connect(&mut self, server: Server, ticket: fchat::Ticket, character: String) {
        self.character = Some(character.clone());
        self.server = Some(server.clone());
        let (connection_tx, internal_rx) = futures::sync::mpsc::channel(32);
        self.fchat_tx = Some(connection_tx);
        let handle = self.handle.clone();
        let handle2 = self.handle.clone();
        let event_tx = self.event_tx.clone();
        let connection = fchat::connect(server, handle)
            .and_then(move |(sink, stream)| {
                (
                    fchat::identify(
                        sink,
                        &ticket,
                        character,
                        "RSFChat".to_owned(),
                        "0.0.1".to_owned(),
                    ),
                    Ok(stream),
                )
            })
            .map_err(|_| ())
            .and_then(move |(sink, stream)| {
                handle2.spawn(
                    stream
                        .map_err(Error::from)
                        .map(Event::ReceivedMessage)
                        .forward(event_tx)
                        .then(|_| Ok(())),
                );
                let timer = Timer::default()
                    .interval(Duration::from_secs(30))
                    .map(|_| fchat::message::client::Message::PIN)
                    .map_err(|_| ());
                sink.sink_map_err(|_| ())
                    .send_all(timer.select(internal_rx))
                    .then(|_| Ok(()))
            });
        self.handle.spawn(connection);
    }
}

fn step(
    mut controller: NetworkController,
    event: Event,
) -> Box<Future<Item = NetworkController, Error = Error>> {
    match event {
        Event::Connect {
            server,
            ticket,
            character,
        } => {
            controller.connect(server, ticket, character);
        }
        Event::ReceivedMessage(message) => {
            controller
                .ui_sender
                .send(ui::Event::ReceivedMessage(message))
                .expect("Failed to send message to UI");
        }
        Event::SendMessage(message) => {
            if let Some(fchat_tx) = controller.fchat_tx.take() {
                let future = fchat_tx
                    .send(message)
                    .map_err(Error::from)
                    .and_then(|sink| {
                        controller.fchat_tx = Some(sink);
                        Ok(controller)
                    });
                return Box::new(future);
            } else {
                panic!("Tried to send message, but not connected to the server")
            }
        }
    }
    Box::new(futures::future::ok(controller))
}

pub fn start() -> Result<(), Error> {
    let (event_tx, event_rx) = futures::sync::mpsc::unbounded();
    let ui_sender = ui::start(event_tx.clone());
    let mut core = Core::new().map_err(Error::from)?;
    let controller = NetworkController::new(core.handle(), ui_sender, event_tx);
    let future = event_rx
        .map_err(|_| format_err!("event_rx error"))
        .fold(controller, step)
        .map(|_| ());
    core.run(future)
}
