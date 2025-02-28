use std::time::Duration;

use fchat::{self, ClientMessage, Server, ServerMessage, Ticket};

use futures::{StreamExt, prelude::*};

use miette::IntoDiagnostic;
use stream::TryStreamExt;
use tokio::sync::mpsc::{Sender, channel, unbounded_channel};
use tokio::time::interval;

use crate::app::{AppError, AppEvent, EventStream};

pub struct ChatController {
    sender: Sender<IoRequest>,
}

impl ChatController {
    pub fn get_ticket(&self, username: String, password: String) {
        self.send(IoRequest::GetTicket { username, password });
    }

    pub fn connect(&self, ticket: Ticket, character: String) {
        self.send(IoRequest::Connect { ticket, character });
    }

    fn send(&self, request: IoRequest) {
        self.sender
            .blocking_send(request)
            .expect("ChatController: IoRequest send failed");
    }
}

pub type RequestSender = Sender<IoRequest>;

pub enum IoRequest {
    GetTicket { username: String, password: String },
    Connect { ticket: Ticket, character: String },
}

pub fn start() -> miette::Result<(ChatController, EventStream)> {
    let (event_sender, event_receiver) = unbounded_channel();
    let (request_sender, mut request_receiver) = channel(16);
    std::thread::Builder::new()
        .name(String::from("io-thread"))
        .spawn(move || {
            let runtime = tokio::runtime::Runtime::new().into_diagnostic().unwrap();
            runtime.block_on(async {
                {
                    let event_sender = event_sender.clone();
                    tokio::spawn(async move {
                        let mut event_stream = crossterm::event::EventStream::new();
                        while let Some(event) = event_stream.next().await {
                            event_sender.send(AppEvent::Crossterm(event)).unwrap();
                        }
                    });
                }

                while let Some(request) = request_receiver.recv().await {
                    match request {
                        IoRequest::GetTicket { username, password } => {
                            let ticket = Ticket::request(&username, &password).await;
                            event_sender.send(AppEvent::Ticket(ticket)).unwrap();
                        }
                        IoRequest::Connect { ticket, character } => {
                            match connect(None, ticket, character).await {
                                Ok((sender, mut stream)) => {
                                    event_sender.send(AppEvent::Connected(sender)).unwrap();
                                    let event_sender = event_sender.clone();
                                    tokio::spawn(async move {
                                        loop {
                                            match stream.try_next().await {
                                                Ok(None) => {
                                                    // TODO: Figure out if I need to do anything here
                                                    break;
                                                }
                                                Ok(Some(message)) => {
                                                    event_sender
                                                        .send(AppEvent::Chat(message))
                                                        .unwrap();
                                                }
                                                Err(error) => {
                                                    event_sender
                                                        .send(AppEvent::Error(
                                                            AppError::Connection(error),
                                                        ))
                                                        .unwrap();
                                                }
                                            }
                                        }
                                    });
                                }
                                Err(error) => {
                                    event_sender.send(AppEvent::Error(AppError::Connection(error)));
                                }
                            }
                        }
                    }
                }
            });
        })
        .into_diagnostic()?;
    let controller = ChatController {
        sender: request_sender,
    };
    Ok((controller, event_receiver))
}

async fn connect(
    server: Option<Server>,
    ticket: Ticket,
    character: String,
) -> Result<
    (
        tokio::sync::mpsc::Sender<ClientMessage>,
        impl Stream<Item = Result<ServerMessage, fchat::Error>>,
    ),
    fchat::Error,
> {
    let mut connection =
        fchat::Connection::connect(server.as_ref().unwrap_or(&Server::Normal)).await?;
    connection
        .identify(
            &ticket,
            character,
            "RSFChat".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        )
        .await?;
    let (mut sink, stream) = connection.split();
    // The sink isn't cloneable, but channel senders are
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    let tx2 = tx.clone();
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            sink.send(message).await.unwrap();
        }
    });
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            let Ok(()) = tx.send(ClientMessage::PIN).await else {
                return;
            };
        }
    });
    Ok((tx2, stream))
}
