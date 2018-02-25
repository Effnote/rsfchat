use cursive::{self, views, Cursive};
use fchat::{self, Ticket};

use futures::sync::mpsc::UnboundedSender;

use std;
use std::thread::spawn;
use std::sync::mpsc::{channel, Receiver, Sender};

use io;

pub enum Event {
    NetEvent(io::Event),
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
    is_running: bool,
    state: State,
    siv: Cursive,
    net_tx: UnboundedSender<io::Event>,
    event_tx: Sender<Event>,
    event_rx: Receiver<Event>,
}

impl Controller {
    fn new(
        net_tx: UnboundedSender<io::Event>,
        event_tx: Sender<Event>,
        event_rx: Receiver<Event>,
    ) -> Controller {
        let mut siv = cursive::Cursive::new();
        siv.set_fps(30);
        Controller {
            is_running: true,
            siv,
            net_tx,
            event_tx,
            event_rx,
            state: State::Start,
        }
    }

    fn step(&mut self) {
        use std::sync::mpsc::TryRecvError;
        let mut next_state = None;
        match self.state {
            State::Start => {
                let (login_tx, login_rx) = channel();
                self.siv.add_layer(login_dialog(login_tx));
                next_state = Some(State::Login(login_rx));
            }
            State::Login(ref login_rx) => match login_rx.try_recv() {
                Ok((ticket, character)) => {
                    self.net_tx
                        .unbounded_send(io::Event::Connect {
                            ticket,
                            character,
                            server: fchat::Server::Debug,
                        })
                        .unwrap();
                    next_state = Some(State::Connecting);
                }
                Err(TryRecvError::Empty) => {},
                Err(TryRecvError::Disconnected) => unimplemented!(),
            },
            _ => unimplemented!(),
        }
        if let Some(state) = next_state {
            self.state = state;
        }
        self.siv.step();
    }

    fn is_running(&self) -> bool {
        self.is_running
    }
}

pub fn login_dialog(result: Sender<(Ticket, String)>) -> views::Dialog {
    use cursive::view::{Boxable, Identifiable};
    let username = views::LinearLayout::horizontal()
        .child(views::TextView::new("Username"))
        .child(views::DummyView)
        .child(views::EditView::new().with_id("username").min_width(30));
    let password = views::LinearLayout::horizontal()
        .child(views::TextView::new("Password"))
        .child(views::DummyView)
        .child(
            views::EditView::new()
                .secret()
                .with_id("password")
                .min_width(30),
        );
    let inputs = views::LinearLayout::vertical()
        .child(username)
        .child(password);
    views::Dialog::around(inputs)
        .button("Login", move |siv| select_character(siv, result.clone()))
        .button("Quit", |siv| siv.quit())
}

fn select_character(siv: &mut Cursive, result: Sender<(Ticket, String)>) {
    let username = siv.call_on_id("username", |text: &mut views::EditView| text.get_content())
        .expect("Failed to find ID \"username\"");
    let password = siv.call_on_id("password", |text: &mut views::EditView| text.get_content())
        .expect("Failed to find ID \"password\"");
    let ticket = Ticket::request(&username, &password).unwrap();
    siv.pop_layer();
    let mut characters = views::SelectView::new();
    characters.add_all_str(ticket.characters().iter().cloned());
    characters.set_on_submit::<_, str>(move |siv, character| {
        result
            .send((ticket.clone(), String::from(character)))
            .unwrap();
        siv.pop_layer();
    });
    siv.add_layer(characters);
}

pub fn start(net_tx: UnboundedSender<io::Event>) -> Sender<Event> {
    let (event_tx, event_rx) = std::sync::mpsc::channel();
    let event_tx2 = event_tx.clone();
    spawn(move || {
        let mut controller = Controller::new(net_tx, event_tx, event_rx);
        while controller.is_running() {
            controller.step();
        }
    });
    event_tx2
}
