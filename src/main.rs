mod error;
mod io;
mod ui;

#[tokio::main]
pub async fn main() {
    let mut siv = ui::init().await;
    siv.run();
}
