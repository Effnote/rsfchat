use std::sync::mpsc::Sender;

pub fn get_input(tx: Sender<String>) {
    let mut stdin = ::std::io::stdin();
    loop {
        tx.send(stdin.read_line().unwrap()).unwrap();
    }
}

pub fn parse(line: &str) -> Action {
    if &line[..1] != "/" {
        Action::Message { content: line }
    } else {
        if let Some(action) = line[1..].split(' ').nth(0) {
            match action {
                "join" => {
                    if let Some(room) = line.splitn(1, ' ').nth(1) {
                        Action::Join { room: room }
                    } else {
                        Action::Error { error: "/join expects a room name." }
                    }
                }
                _ => Action::Invalid { action: action }
            }
        } else { Action::None }
    }
}

pub enum Action<'a> {
    Message {
        content: &'a str,
    },
    Join {
        room: &'a str,
    },
    Priv {
        character: &'a str,
    },
    Error {
        error: &'static str
    },
    Invalid {
        action: &'a str
    },
    None,
}