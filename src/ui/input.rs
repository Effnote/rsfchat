use std::sync::mpsc::Sender;

pub fn get_input(tx: Sender<String>) {
    let mut stdin = ::std::old_io::stdin();
    loop {
        let mut line = stdin.read_line().unwrap();
        if line.chars().rev().nth(0).map_or(false, |x| x == '\n') {
            line.pop();
        }
        tx.send(line).unwrap();
    }
}

pub fn parse(line: &str) -> Action {
    if &line[..1] != "/" {
        Action::Message { content: line }
    } else {
        if let Some(action) = line[1..].split(' ').nth(0) {
            match action {
                "priv" => {
                    if let Some(character) = line.splitn(1, ' ').nth(1) {
                        Action::Priv { character: character }
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
