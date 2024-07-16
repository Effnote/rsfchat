use cursive::traits::Nameable;
use cursive::view::Resizable;
use cursive::views::{EditView, ViewRef};
use cursive::CursiveRunnable;
use cursive::{self, views, Cursive};

fn login_dialog() -> views::Dialog {
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
        .button("Login", |siv| {
            let username: ViewRef<EditView> = siv.find_name("username").unwrap();
            let password: ViewRef<EditView> = siv.find_name("password").unwrap();
            let username = username.get_content().to_string();
            let password = password.get_content().to_string();

            let cb_sink = siv.cb_sink().clone();
            tokio::runtime::Handle::current().spawn(async move {
                if let Ok(ticket) = fchat::Ticket::request(&username, &password).await {
                    let characters = ticket.characters().to_vec();
                    cb_sink
                        .send(Box::new(move |siv| select_character(siv, characters)))
                        .unwrap();
                } else {
                    cb_sink
                        .send(Box::new(|siv| show_error(siv, "Failed to log in.")))
                        .unwrap();
                }
            });
        })
        .button("Quit", |siv| siv.quit())
}

fn show_error(siv: &mut Cursive, error: &str) {
    let dialog = views::Dialog::new()
        .content(views::TextView::new(error))
        .button("Ok", |siv| {
            siv.pop_layer();
        })
        .with_name("error");
    siv.add_layer(dialog)
}

pub fn select_character(siv: &mut Cursive, character_list: Vec<String>) {
    siv.pop_layer();
    let mut characters = views::SelectView::new();
    characters.add_all_str(character_list);
    characters.set_on_submit::<_, (), str>(move |siv, _character| {
        siv.pop_layer();
    });
    siv.add_layer(characters);
}

pub async fn init() -> CursiveRunnable {
    let mut siv = cursive::default();
    siv.set_fps(30);
    siv.add_layer(login_dialog());
    siv
}
