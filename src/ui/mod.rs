use cursive::Cursive;

use fchat::{self, Ticket};

use futures::channel::mpsc::UnboundedSender;

use std;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::spawn;

use crate::io;

mod backend;
use backend::*;

pub enum Event {
    ReceivedMessage(fchat::message::server::Message),
    TextInput(String),
}

enum State {
    Start,
    Login(Receiver<(Ticket, String)>),
    Connecting,
    Connected,
    Disconnected,
    Quit,
}

struct Controller {
    state: State,
    net_tx: UnboundedSender<io::Event>,
    siv_tx: cursive::CbSink,
    event_tx: Sender<Event>,
    event_rx: Receiver<Event>,
}

impl Controller {
    fn new() -> Controller {
        Controller {
            state: State::Start,
        }
    }

    fn step(&mut self) -> Option<()> {
        use std::sync::mpsc::TryRecvError;
        let mut next_state = None;
        match self.state {
            State::Start => {
                let (login_tx, login_rx) = channel();
                self.siv_tx.send(Box::new(|siv: &mut Cursive| {
                    siv.add_layer(login_dialog(login_tx))
                }));
                next_state = Some(State::Login(login_rx));
            }
            State::Login(ref login_rx) => match login_rx.try_recv() {
                Ok((ticket, character)) => {
                    self.net_tx
                        .unbounded_send(io::Event::Connect {
                            ticket,
                            character,
                            server: fchat::Server::Normal,
                        })
                        .unwrap();
                    next_state = Some(State::Connecting);
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    panic!("ui login_rx disconnected");
                }
            },
            State::Connecting => {
                std::thread::sleep(std::time::Duration::from_secs(5));
                let message = fchat::message::client::Message::JCH {
                    channel: String::from("Development"),
                };
                self.net_tx
                    .unbounded_send(io::Event::SendMessage(message))
                    .unwrap();
                let event_tx = self.event_tx.clone();
                self.siv_tx
                    .send(Box::new(move |siv: &mut Cursive| debug_view(siv, event_tx)));
                next_state = Some(State::Connected);
            }
            State::Connected => match self.event_rx.try_recv() {
                Ok(Event::ReceivedMessage(message)) => {
                    self.siv_tx.send(Box::new(move |siv: &mut Cursive| {
                        debug_message(siv, message)
                    }));
                }
                Ok(Event::TextInput(text)) => {
                    let message = fchat::message::client::Message::MSG {
                        channel: String::from("Development"),
                        message: text.clone(),
                    };
                    self.net_tx
                        .unbounded_send(io::Event::SendMessage(message))
                        .unwrap();
                    self.siv_tx
                        .send(Box::new(move |siv: &mut Cursive| debug_message(siv, text)));
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    panic!("ui event_rx disconnected");
                }
            },
            State::Disconnected => {
                panic!("State::Disconnected");
            }
            State::Quit => {
                return None;
            }
        }
        if let Some(state) = next_state {
            self.state = state;
        }
        Some(())
    }
}

pub fn start(net_tx: UnboundedSender<io::Event>) -> Sender<Event> {
    let controller = Controller::new();
    let (event_tx, event_rx) = std::sync::mpsc::channel();
    let event_tx2 = event_tx.clone();
    spawn(move || -> Result<(), std::io::Error> { Ok(()) });
    event_tx2
}
