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

macro_rules! impl_msg {
    ($name: ident) => {
        impl<'a> ::message::ClientMessage for $name<'a> {
            fn send(self, client: &mut ::websocket::client::WebSocketLocalClient) {
                let message = format!("{} {}", stringify!($name), ::rustc_serialize::json::encode(&self));
                client.send_message(::websocket::WebSocketMessage::Text(message)).unwrap();
            }
        }
    }
}

pub mod out {
    #[derive(RustcEncodable)]
    pub struct IDN<'a> {
        pub method: &'a str,
        pub account: &'a str,
        pub ticket: &'a str,
        pub character: &'a str,
        pub cname: &'a str,
        pub cversion: &'a str,
    }

    impl_msg!(IDN);
}
