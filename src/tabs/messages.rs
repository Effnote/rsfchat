use std::borrow::ToOwned;

use config;
use time;
use super::dropping_buf::DroppingBuf;

pub struct Stamp {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
}

impl Stamp {
    fn new() -> Stamp {
        let time = time::now();
        Stamp {
            hours: time.tm_hour as u8,
            minutes: time.tm_min as u8,
            seconds: time.tm_sec as u8,
        }
    }
}

pub struct Message {
    pub timestamp: Stamp,
    pub contents: String,
    pub kind: MessageType,
}

pub enum MessageType {
    Normal {
        from: String,
    },
    Me {
        from: String,
    },
    System,
}

pub struct Log {
    pub messages: Vec<Message>,
    pub display_buffer: DroppingBuf<String>,
    formatter: Box<Fn(&Message) -> String + 'static>,
}

impl Log {
    pub fn new(buffer_size: u16, formatter: Box<Fn(&Message) -> String + 'static>) -> Log {
        Log {
            messages: vec![],
            display_buffer: DroppingBuf::with_capacity(buffer_size),
            formatter: formatter,
        }
    }

    pub fn add_message(&mut self, kind: MessageType, message: &str) {
        let message = Message {
            timestamp: Stamp::new(),
            contents: message.to_owned(),
            kind: kind,
        };
        self.display_buffer.insert((self.formatter)(&message));
        self.messages.push(message);
    }

    pub fn resize(&mut self, new_size: u16) {
        let Log { ref mut display_buffer, ref formatter, .. } = *self;
        display_buffer.resize(new_size, self.messages.iter().map(|x| formatter(x)));
    }
}
