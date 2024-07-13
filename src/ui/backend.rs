use chrono::{self, Timelike};
use cursive::traits::{Nameable, Scrollable};
use cursive::view::Resizable;
use cursive::{self, views, Cursive};

use fchat::{self, Ticket};

use std::sync::mpsc::Sender;

use super::Event;

pub struct Handle {
    sender: cursive::CbSink,
}

impl Handle {}

pub fn login_dialog(result: Sender<(Ticket, String)>) -> views::Dialog {
    let username = views::LinearLayout::horizontal()
        .child(views::TextView::new("Username"))
        .child(views::DummyView)
        .child(views::EditView::new().with_name("username").min_width(30));
    let password = views::LinearLayout::horizontal()
        .child(views::TextView::new("Password"))
        .child(views::DummyView)
        .child(
            views::EditView::new()
                .secret()
                .with_name("password")
                .min_width(30),
        );
    let inputs = views::LinearLayout::vertical()
        .child(username)
        .child(password);
    views::Dialog::around(inputs)
        .button("Login", move |siv| select_character(siv, result.clone()))
        .button("Quit", |siv| siv.quit())
}

pub fn select_character(siv: &mut Cursive, result: Sender<(Ticket, String)>) {
    let username = siv
        .call_on_name("username", |text: &mut views::EditView| text.get_content())
        .expect("Failed to find ID \"username\"");
    let password = siv
        .call_on_name("password", |text: &mut views::EditView| text.get_content())
        .expect("Failed to find ID \"password\"");
    let ticket = Ticket::request(&username, &password).unwrap();
    siv.pop_layer();
    let mut characters = views::SelectView::new();
    characters.add_all_str(ticket.characters().iter().cloned());
    characters.set_on_submit::<_, (), str>(move |siv, character| {
        result
            .send((ticket.clone(), String::from(character)))
            .unwrap();
        siv.pop_layer();
    });
    siv.add_layer(characters);
}

pub fn debug_view(siv: &mut Cursive, event_tx: Sender<Event>) {
    let textview = views::TextView::empty()
        .with_name("debug_view")
        .scrollable()
        .scroll_strategy(cursive::view::ScrollStrategy::StickToBottom)
        .full_screen();
    let input = views::EditView::new()
        .on_submit(move |siv, text| {
            event_tx.send(Event::TextInput(String::from(text))).unwrap();
            siv.call_on_name("input", |input: &mut views::EditView| input.set_content(""));
        })
        .with_name("input");
    let layer = views::LinearLayout::vertical()
        .child(textview)
        .child(input)
        .full_screen();
    siv.add_layer(layer);
}

pub fn debug_message<M: std::fmt::Debug>(siv: &mut Cursive, message: M) {
    siv.call_on_name("debug_view", |view: &mut views::TextView| {
        let now = chrono::Local::now();
        let hour = now.hour();
        let minute = now.minute();
        let second = now.second();
        view.append(format!(
            "[{:02}:{:02}:{:02}] {:?}\n",
            hour, minute, second, message
        ));
    });
}
