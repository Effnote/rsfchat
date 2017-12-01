use cursive::{self, Cursive, views};
use fchat::Ticket;

use futures::sync::oneshot;

use std;
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::sync::mpsc;

pub type UiSender = std::sync::mpsc::Sender<Box<Fn(&mut Cursive) + Send>>;

pub type LoginDataSender = Arc<Mutex<Option<oneshot::Sender<(Ticket, String)>>>>;

pub fn login_dialog(result: LoginDataSender) -> views::Dialog {
    use cursive::view::{Boxable, Identifiable};
    let username = views::LinearLayout::horizontal()
        .child(views::TextView::new("Username"))
        .child(views::DummyView)
        .child(views::EditView::new().with_id("username").min_width(30));
    let password = views::LinearLayout::horizontal()
        .child(views::TextView::new("Password"))
        .child(views::DummyView)
        .child(views::EditView::new().secret().with_id("password").min_width(30));
    let inputs = views::LinearLayout::vertical()
        .child(username)
        .child(password);
    views::Dialog::around(inputs)
        .button("Login", move |siv| select_character(siv, result.clone()))
        .button("Quit", |siv| siv.quit())
}

fn select_character(siv: &mut Cursive, result: LoginDataSender) {
    let username = siv.call_on_id("username", |text: &mut views::EditView| text.get_content()).expect("Failed to find ID \"username\"");
    let password = siv.call_on_id("password", |text: &mut views::EditView| text.get_content()).expect("Failed to find ID \"password\"");
    let ticket = Ticket::request(&username, &password).unwrap();
    siv.pop_layer();
    let mut characters = views::SelectView::new();
    characters.add_all_str(ticket.characters().iter().cloned());
    characters.set_on_submit::<_, str>(
        move |siv, character| {
            let result = result.lock().unwrap().take().expect("select_character result is None");
            result.send((ticket.clone(), String::from(character))).unwrap();
            siv.pop_layer();
        }
    );
    siv.add_layer(characters);
}

pub fn start() -> UiSender {
    let (tx, rx) = mpsc::channel();
    spawn(move || {
        let mut siv = cursive::Cursive::new();
        tx.send(siv.cb_sink().clone());
        siv.set_fps(30);
        siv.run();
    });
    rx.recv().unwrap()
}
