use std::sync::mpsc::{Receiver, channel};
use std::thread;
use std::borrow::ToOwned;

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
    use tabs::messages::Message;
    let formatter = || -> Box<Fn(&Message) -> String> { Box::new(|message| format!("{}", message.contents)) }; // TODO
    let mut ui_data = UI {
        open_tabs: tabs::Tabs::new(&config, formatter),
        config: config,
        ticket: ticket,
        sender: sender,
    };
    out::IDN {
        method: "ticket",
        account: &*ui_data.config.user_info.username,
        ticket: &*ui_data.ticket.ticket,
        character: &*ui_data.config.user_info.character,
        cname: "RSFChat",
        cversion: "0.0.1"
    }.send(&mut ui_data.sender);

    let (input_tx, input_rx) = channel();
    thread::spawn(move|| input::get_input(input_tx));

    loop {
        select! {
            line = input_rx.recv() => {
                perform(&mut ui_data, line.unwrap());
            },
            msg = rx.recv() => {
                let msg = msg.unwrap();
                ui_data.open_tabs.dispatch(msg);
            }
        }
        refresh(&mut ui_data);
    }
}

fn refresh(ui: &mut UI) {
    let current_tab = ui.open_tabs.get_current().unwrap();
    let text = &current_tab.get_log().display_buffer;
    for i in text.len().saturating_sub(12)..text.len() {
        println!("{}", text[i]);
    }
}

fn perform(ui: &mut UI, line: String) {
    use ui::input::Action::*;
    use tabs::messages::MessageType::*;
    let UI { ref mut open_tabs, ref mut sender, ..} = *ui;
    match input::parse(&*line) {
        Message { mut content } => {
            let target = open_tabs.get_current();
            if let Err(err) = target {
                println!("Error: {}", err);
                return;
            }
            let mut target = target.unwrap();
            target.send_message(sender, Normal { from: ui.config.user_info.character.to_owned() }, content)
                .err().map(|e| println!("Error: {}", e));
        }
        Join { room } => {
            open_tabs.add_tab(room, false);
        }
        Priv { character } => {
            open_tabs.add_tab(character, true);
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
