use ui;
use ui::UiSender;
use config::Config;

use std;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use fchat::{self, Server};

use tokio_core::reactor::Core;
use tokio_timer::{self, Timer};

use futures::{self, Future, Stream, Sink};

type ControllerSender = futures::sync::mpsc::Sender<Event>;
type BoxedSender = Box<Sink<SinkItem = Event, SinkError = Error>>;

struct Controller {
    config: Config,
    ui_sender: UiSender,
    controller_sender: ControllerSender,
}

impl Controller {
    fn new(config: Config, ui_sender: UiSender, controller_sender: ControllerSender) -> Controller {
        Controller {
            config,
            ui_sender,
            controller_sender,
        }
    }

    fn get_sender(&self) -> BoxedSender {
        Box::new(self.controller_sender.clone().sink_map_err(|_| Error::SendError))
    }

    fn handle(&self, event: Event) -> Result<(), Error> {
        Ok(())
    }  
}

impl Drop for Controller {
    fn drop(&mut self) {
        let _ = self.ui_sender.send(Box::new(|siv| siv.quit()));
        std::thread::sleep(Duration::from_secs(1));
    }
}

pub enum Event {
    Received(fchat::message::server::Message),
    SendRaw(fchat::message::client::Message),
}

pub enum Error {
    FchatError(fchat::Error),
    IoError(std::io::Error),
    SendError,
    TimerError,
    Unknown,
}

impl From<fchat::Error> for Error {
    fn from(error: fchat::Error) -> Error {
        Error::FchatError(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Error {
        Error::IoError(error)
    }
}

impl<T> From<futures::sync::mpsc::SendError<T>> for Error {
    fn from(error: futures::sync::mpsc::SendError<T>) -> Error {
        Error::SendError
    }
}

impl From<tokio_timer::TimerError> for Error {
    fn from(error: tokio_timer::TimerError) -> Error {
        Error::TimerError
    }
}

impl From<()> for Error {
    fn from(error: ()) -> Error {
        Error::Unknown
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::FchatError(ref error) => error.fmt(f),
            Error::IoError(ref error) => error.fmt(f),
            Error::SendError => "Send error".fmt(f),
            Error::TimerError => "Timer error".fmt(f),
            Error::Unknown => "Unknown error".fmt(f),
        }
    }
}

pub fn start() -> Result<(), Error> {
    let ui_sender = ui::start();
    let (logindata_tx, logindata_rx) = futures::sync::oneshot::channel();
    let logindata_tx = Arc::new(Mutex::new(Some(logindata_tx)));
    ui_sender.send(Box::new(move |siv| siv.add_layer(ui::login_dialog(logindata_tx.clone())))).unwrap();
    let (controller_tx, controller_rx) = futures::sync::mpsc::channel(32);
    let controller = &Controller::new(Config::default(), ui_sender, controller_tx);
    let mut core = Core::new()?;
    let handle = core.handle();
    let handle2 = core.handle();
    let future = logindata_rx
        .map_err(|_| fchat::Error::Channel)
        .and_then(|(ticket, character)| {
            fchat::connect(Server::Debug, &handle)
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
        })
        .from_err()
        .and_then(move |(sink, stream)| {
            let controller_tx = controller.get_sender();
            handle2.spawn(
                stream.map(Event::Received)
                .map_err(Error::from)
                .forward(controller_tx)
                .then(|_| Ok(()))
            );
            let controller_tx = controller.get_sender();
            handle2.spawn(
                Timer::default()
                .interval(Duration::from_secs(30))
                .map(|_| Event::SendRaw(fchat::message::client::Message::PIN))
                .map_err(Error::from)
                .forward(controller_tx)
                .then(|_| Ok(()))
            );
            controller_rx.from_err().for_each(move |event| controller.handle(event))
        });
    core.run(future)
}
