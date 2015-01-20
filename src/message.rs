use std::sync::mpsc::Sender;

#[derive(Show)]
pub enum ServerMessage {
    Other { kind: [u8; 3], contents: String }
}

pub fn handle(text: String, tx: &Sender<ServerMessage>) {
    let mut kind = [0; 3];
    ::std::slice::bytes::copy_memory(&mut kind, &text.as_bytes()[..3]);
    tx.send(ServerMessage::Other { kind: kind, contents: text });
}

pub trait ClientMessage {
    fn send(self, client: &mut ::websocket::client::WebSocketLocalClient);
}


pub mod out {
    use rustc_serialize::json;
    use websocket::client::WebSocketLocalClient;
    use websocket::WebSocketMessage::Text;

    macro_rules! create_struct {
        ($name: ident, $($fields: ident),+ ) => {
            #[derive(RustcEncodable)]
            pub struct $name<'a> {
                $(
                    pub $fields: &'a str,
                )+
            }

            impl<'a> ::message::ClientMessage for $name<'a> {
                fn send(self, client: &mut WebSocketLocalClient) {
                    let message = format!("{} {}", stringify!($name), json::encode(&self));
                    client.send_message(Text(message)).unwrap();
                }
            }
        }
    }

    create_struct!(IDN, method, account, ticket, character, cname, cversion);
    create_struct!(MSG, channel, message);
    create_struct!(RLL, channel, dice);
}
