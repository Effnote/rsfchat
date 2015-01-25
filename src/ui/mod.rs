use std::sync::mpsc::{Receiver, channel};
use std::thread::Thread;

use message::{ServerMessage, ClientMessage, out};

use config::Config;
use ticket::Ticket;
use tabs;
use WsSender;

mod input;

struct UI {
    open_tabs: tabs::Tabs,
    config: Config,
    ticket: Ticket,
    sender: WsSender,
}

pub fn start(rx: Receiver<ServerMessage>, config: Config, ticket: Ticket, sender: WsSender) {
    let mut ui_data = UI {
        open_tabs: tabs::Tabs::new(),
        config: config,
        ticket: ticket,
        sender: sender,
    };
    out::IDN {
        method: "ticket",
        account: &*ui_data.config.username,
        ticket: &*ui_data.ticket.ticket,
        character: &*ui_data.config.character,
        cname: "RSFChat",
        cversion: "0.0.1"
    }.send(&mut ui_data.sender);

    let (input_tx, input_rx) = channel();
    Thread::spawn(move|| input::get_input(input_tx));

    loop {
        select! {
            line = input_rx.recv() => { perform(&mut ui_data, line.unwrap()) },
            msg = rx.recv() => { println!("{:?}", msg.unwrap()); }
        }
    }
}

fn perform(ui: &mut UI, line: String) {
    use ui::input::Action::*;
    let UI { ref mut open_tabs, ref mut sender, ..} = *ui;
    match input::parse(&*line) {
        Message { content } => {
            let target = open_tabs.get_current();
            target.send_message(sender, content)
                .err().map(|e| println!("Error: {}", e));
        }
        Join { room } => { // TODO
            unimplemented!();
        }
        Priv { character } => { // TODO
            unimplemented!();
        }
        Error { error } => {
            println!("Error: {}", error);
        }
        Invalid { action } => {
            println!("Unknown command: {}", action);
        }
        None => {}
    }
}
