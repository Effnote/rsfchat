use std::time::Duration;

use fchat::message::client::Message as ClientMessage;
use fchat::message::server::Message as ServerMessage;
use fchat::{self, Server, Ticket};

use futures::{prelude::*, StreamExt};

use tokio::time::interval;

pub async fn connect(
    server: Option<Server>,
    ticket: Ticket,
    character: String,
) -> Result<
    (
        tokio::sync::mpsc::Sender<ClientMessage>,
        impl Stream<Item = Result<ServerMessage, fchat::Error>>,
    ),
    fchat::Error,
> {
    let mut connection =
        fchat::Connection::connect(server.as_ref().unwrap_or(&Server::Normal)).await?;
    connection
        .identify(
            &ticket,
            character,
            "RSFChat".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        )
        .await?;
    let (mut sink, stream) = connection.split();
    // The sink isn't cloneable, but channel senders are
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    let tx2 = tx.clone();
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            sink.send(message).await.unwrap();
        }
    });
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            let Ok(()) = tx.send(ClientMessage::PIN).await else {
                return;
            };
        }
    });
    Ok((tx2, stream))
}
