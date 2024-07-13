use std::time::Duration;

use fchat::{self, Server};

use futures::channel::mpsc::{Sender, UnboundedSender};
use futures::prelude::*;

use tokio::time::interval;

use crate::ui;
use crate::Error;

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
    event_tx: UnboundedSender<Event>,
    ui_sender: std::sync::mpsc::Sender<ui::Event>,
    server: Option<Server>,
    fchat_tx: Option<Sender<fchat::message::client::Message>>,
    character: Option<String>,
}

impl NetworkController {
    fn new(
        ui_sender: std::sync::mpsc::Sender<ui::Event>,
        event_tx: UnboundedSender<Event>,
    ) -> NetworkController {
        NetworkController {
            fchat_tx: None,
            event_tx,
            ui_sender,
            server: None,
            character: None,
        }
    }

    fn connect(&mut self, server: Server, ticket: fchat::Ticket, character: String) {
        self.character = Some(character.clone());
        let (connection_tx, internal_rx) = futures::channel::mpsc::channel(32);
        self.fchat_tx = Some(connection_tx);
        let event_tx = self.event_tx.clone();
        self.server = Some(server.clone());
        let connection_future = async move {
            let mut connection = fchat::Connection::connect(&server).await?;
            connection
                .identify(&ticket, character, "RSFChat".to_owned(), "0.0.2".to_owned())
                .await?;
            let (mut sink, stream) = connection.split();
            tokio::spawn(async move {
                event_tx
                    .sink_map_err(Error::from)
                    .send_all(&mut stream.map_ok(Event::ReceivedMessage).map_err(Error::from))
                    .await
            });
            let ping =
                interval(Duration::from_secs(30)).map(|_| fchat::message::client::Message::PIN);
            let mut outgoing_messages = futures::stream::select(internal_rx, ping);
            while let Some(message) = outgoing_messages.next().await {
                sink.send(message).await?;
            }
            Ok::<(), Error>(())
        };
        tokio::spawn(connection_future);
    }
}

async fn step(controller: &mut NetworkController, event: Event) -> Result<(), Error> {
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
            if let Some(ref mut fchat_tx) = controller.fchat_tx {
                fchat_tx.send(message).await?;
            } else {
                panic!("Tried to send message, but not connected to the server")
            }
        }
    }
    Ok(())
}

pub fn start() -> Result<(), Error> {
    let (event_tx, mut event_rx) = futures::channel::mpsc::unbounded();
    let ui_sender = ui::start(event_tx.clone());
    let mut controller = NetworkController::new(ui_sender, event_tx);
    let future = async {
        while let Some(event) = event_rx.next().await {
            step(&mut controller, event).await?;
        }
        Ok(())
    };
    let mut runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(future)
}
