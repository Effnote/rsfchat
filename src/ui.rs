use cursive::{self, Cursive, views};
use fchat::Ticket;

fn login_dialog() -> views::Dialog {
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
        .button("Login", |siv| select_character(siv))
        .button("Quit", |siv| siv.quit())
}

fn select_character(siv: &mut Cursive) {
    let username = siv.call_on_id("username", |text: &mut views::EditView| text.get_content()).expect("Failed to find ID \"username\"");
    let password = siv.call_on_id("password", |text: &mut views::EditView| text.get_content()).expect("Failed to find ID \"password\"");
    let ticket = Ticket::request(&username, &password).unwrap();
    siv.pop_layer();
    let mut characters = views::SelectView::new();
    characters.add_all_str(ticket.characters().iter().cloned());
    characters.set_on_submit::<_, str>(|siv, character| {
        siv.pop_layer();
    });
    siv.add_layer(characters);
}

pub fn start() {
    let mut siv = cursive::Cursive::new();
    siv.set_fps(30);
    siv.add_layer(login_dialog());
    siv.run();
}
