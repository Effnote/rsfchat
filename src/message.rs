use std::sync::mpsc::Sender as CSender;
use WsSender;

#[derive(Debug)]
pub enum ServerMessage {
    Other { kind: [u8; 3], contents: String },
}

pub fn handle(text: String, tx: &CSender<ServerMessage>) {
    let mut kind = [0; 3];
    for (i, &x) in text.as_bytes()[..3].iter().enumerate() {
        kind[i] = x;
    }
    tx.send(ServerMessage::Other { kind: kind, contents: text }).unwrap();
}

pub trait ClientMessage {
    fn send(self, sender: &mut WsSender);
}

#[allow(dead_code)]
pub mod out {
    use rustc_serialize::json;
    use WsSender;

    macro_rules! create_struct {
        ($name: ident, $($fields: ident),+ ) => {
            #[must_use]
            #[derive(RustcEncodable)]
            pub struct $name<'a> {
                $(
                    pub $fields: &'a str,
                )+
            }

            impl<'a> ::message::ClientMessage for $name<'a> {
                fn send(self, sender: &mut WsSender) {
                    let message = format!("{} {}", stringify!($name), json::encode(&self).unwrap());
                    sender.send(message).unwrap();
                }
            }
        }
    }

    create_struct!(IDN, method, account, ticket, character, cname, cversion);
    create_struct!(MSG, channel, message);
    create_struct!(PRI, recipient, message);
    create_struct!(RLL, channel, dice);
}
