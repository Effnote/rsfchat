use message::{ClientMessage, out};
use WsSender;

pub struct Tab {
    pub joined: bool,
    pub name: String,
    pub kind: TabType,
    pub open: bool,
}

impl Tab {
    pub fn send_message(&self, sender: &mut WsSender, message: &str) -> Result<(), &'static str> {
        use self::TabType::*;
        match self.kind {
            Status => return Err("Can't send a message into the status window."),
            Channel(ref name) => {
                out::MSG {
                    channel: &**name,
                    message: message
                }.send(sender);
            }
            Private(ref name) => {
                out::PRI {
                    recipient: &**name,
                    message: message
                }.send(sender);
            }
        }
        Ok(())
    }
}

pub enum TabType {
    Status,
    Channel(String),
    Private(String),
}
