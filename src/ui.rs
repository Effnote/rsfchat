use message::{ServerMessage, ClientMessage};

use config::Config;
use ticket::Ticket;
use message::out;
use websocket::client::WebSocketLocalClient;

pub fn start(rx: Receiver<ServerMessage>, config: Config, ticket: Ticket, mut client: WebSocketLocalClient) {
    out::IDN {
        method: "ticket",
        account: &*config.username,
        ticket: &*ticket.ticket,
        character: &*config.character,
        cname: "RSFChat",
        cversion: "0.0.1"
    }.send(&mut client);

    for msg in rx.iter() {
        println!("{}", msg);
    }
}
