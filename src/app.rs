use crokey::key;
use crossterm::event::{Event, KeyEvent};
use fchat::{ClientMessage, ServerMessage, Ticket};
use miette::IntoDiagnostic;
use ratatui::{
    DefaultTerminal,
    text::Text,
    widgets::{List, ListState, Paragraph},
};
use ratatui_macros::vertical;
use ringbuffer::{AllocRingBuffer, RingBuffer};
use std::io;
use tokio::sync::mpsc::{Sender, UnboundedReceiver};
use tui_prompts::{FocusState, Prompt, State, TextPrompt, TextRenderStyle, TextState};

use crate::io::ChatController;
use crate::widgets::{TextArea, TextAreaState};

pub type EventStream = UnboundedReceiver<AppEvent>;

pub struct App {
    state: AppScreen,
    needs_redraw: bool,
    pub should_quit: bool,
    chat_controller: ChatController,
    character: String,
    debug_data: AllocRingBuffer<String>,
    sender: Option<Sender<ClientMessage>>,
    last_event: Option<Event>,
}

impl App {
    pub fn new(chat_controller: ChatController) -> Self {
        App {
            state: AppScreen::Login {
                focus: 0,
                username: TextState::new().with_focus(FocusState::Focused),
                password: TextState::new().with_focus(FocusState::Unfocused),
            },
            needs_redraw: true,
            should_quit: false,
            chat_controller,
            character: String::new(),
            debug_data: AllocRingBuffer::new(16),
            sender: None,
            last_event: None,
        }
    }

    pub fn draw(&mut self, terminal: &mut DefaultTerminal) -> miette::Result<()> {
        if self.needs_redraw {
            self.needs_redraw = false;
            terminal
                .draw(|frame| {
                    let [debug_area, main_area, status_area] =
                        vertical![*=2, *=1, ==1].areas(frame.area());
                    frame.render_widget(
                        Paragraph::new(
                            self.debug_data
                                .iter()
                                .map(|debug_string| &debug_string[..])
                                .collect::<Text>(),
                        ),
                        debug_area,
                    );
                    frame.render_widget(format!("{:?}", self.last_event), status_area);
                    match &mut self.state {
                        AppScreen::Login {
                            username, password, ..
                        } => {
                            let [username_area, password_area] =
                                vertical![==1, ==1].areas(main_area);
                            TextPrompt::new("Username".into()).draw(frame, username_area, username);
                            TextPrompt::new("Password".into())
                                .with_render_style(TextRenderStyle::Password)
                                .draw(frame, password_area, password);
                        }
                        AppScreen::Characters { ticket, list_state } => {
                            frame.render_stateful_widget(
                                List::new(ticket.characters.clone()).highlight_symbol("> "),
                                main_area,
                                list_state,
                            );
                        }
                        AppScreen::Chat { text_state, .. } => {
                            frame.render_stateful_widget_ref(
                                TextArea::new(),
                                main_area,
                                text_state,
                            );
                        }
                    };
                })
                .into_diagnostic()?;
        }
        Ok(())
    }

    pub fn event(&mut self, event: AppEvent) -> miette::Result<()> {
        self.needs_redraw = true;
        self.debug_data.push(format!("{:?}", event));
        match event {
            AppEvent::Crossterm(event) => {
                let event = event.unwrap();
                self.last_event = Some(event.clone());
                match event {
                    Event::Key(event) => self.key(event),
                    Event::Paste(data) => self.paste(data),
                    _ => {}
                }
            }
            AppEvent::Debug(debug_msg) => self.debug_data.push(debug_msg),
            AppEvent::Chat(_) => {}
            // TODO: Handle ticket errors
            AppEvent::Ticket(ticket) => {
                // TODO: Handle ticket errors
                let Ok(ticket) = ticket else {
                    return Ok(());
                };
                match &mut self.state {
                    AppScreen::Login { .. } => {
                        self.state = AppScreen::Characters {
                            ticket,
                            list_state: ListState::default(),
                        };
                    }
                    AppScreen::Characters {
                        ticket: character_ticket,
                        ..
                    } => {
                        *character_ticket = ticket;
                    }
                    AppScreen::Chat {
                        ticket: chat_ticket,
                        ..
                    } => {
                        *chat_ticket = ticket;
                    }
                }
            }
            AppEvent::Connected(sender) => {
                self.state = match &self.state {
                    AppScreen::Login { .. } => panic!("Connected, but still on Login screen!"),
                    AppScreen::Characters { ticket, .. } => AppScreen::Chat {
                        ticket: ticket.clone(),
                        text_state: TextAreaState::new(),
                    },
                    // TODO: What to do in this case?
                    AppScreen::Chat { .. } => return Ok(()),
                };
                self.sender = Some(sender);
            }
            AppEvent::Error(_) => todo!(),
        }
        Ok(())
    }

    pub fn key(&mut self, event: KeyEvent) {
        let key = event.into();
        match key {
            key!(ctrl - q) => {
                self.should_quit = true;
            }
            key!(shift - tab) => self.focus_prev(),
            key!(tab) => self.focus_next(),
            key!(enter) => match &mut self.state {
                AppScreen::Login {
                    username, password, ..
                } => {
                    let username = username.value().to_owned();
                    let password = password.value().to_owned();
                    let _ = self.chat_controller.get_ticket(username, password);
                }
                AppScreen::Characters { ticket, list_state } => {
                    let Some(selected) = list_state.selected() else {
                        return;
                    };
                    let character = ticket.characters[selected].clone();
                    let ticket = ticket.clone();
                    let _ = self.chat_controller.connect(ticket, character);
                }
                AppScreen::Chat { text_state, .. } => text_state.event(&Event::Key(event)),
            },
            _ => match &mut self.state {
                AppScreen::Login {
                    focus,
                    username,
                    password,
                } => {
                    if *focus == 0 {
                        username.handle_key_event(event);
                    } else {
                        password.handle_key_event(event);
                    }
                }
                AppScreen::Characters { list_state, .. } => match key {
                    key!(up) | key!(left) => {
                        list_state.select_previous();
                    }
                    key!(down) | key!(right) => {
                        list_state.select_next();
                    }
                    _ => {}
                },
                AppScreen::Chat { text_state, .. } => text_state.event(&Event::Key(event)),
            },
        }
    }

    pub fn paste(&mut self, data: String) {
        match &mut self.state {
            AppScreen::Login {
                focus,
                username,
                password,
            } => {
                if *focus == 0 {
                    username.value_mut().push_str(&data);
                } else {
                    password.value_mut().push_str(&data);
                }
            }
            AppScreen::Characters { .. } => {}
            AppScreen::Chat { .. } => todo!(),
        }
    }

    fn focus_prev(&mut self) {
        match &mut self.state {
            AppScreen::Login { focus, .. } => {
                *focus = (*focus + 1) % 2;
            }
            AppScreen::Characters { list_state, .. } => {
                list_state.select_previous();
            }
            AppScreen::Chat { .. } => {}
        }
        self.update_focused();
    }

    fn focus_next(&mut self) {
        match &mut self.state {
            AppScreen::Login { focus, .. } => {
                *focus = (*focus + 1) % 2;
            }
            AppScreen::Characters { list_state, .. } => {
                list_state.select_next();
            }
            AppScreen::Chat { .. } => {}
        }
        self.update_focused();
    }

    fn update_focused(&mut self) {
        match &mut self.state {
            AppScreen::Login {
                focus,
                username,
                password,
            } => {
                if *focus == 0 {
                    username.focus();
                    password.blur();
                } else {
                    username.blur();
                    password.focus();
                }
            }
            AppScreen::Characters { .. } => {}
            AppScreen::Chat { .. } => {}
        }
    }
}

#[derive(Debug)]
pub enum AppEvent {
    Crossterm(Result<crossterm::event::Event, io::Error>),
    Debug(String),
    Ticket(Result<Ticket, fchat::ticket::Error>),
    Connected(tokio::sync::mpsc::Sender<fchat::message::client::Message>),
    Chat(ServerMessage),
    Error(AppError),
}

#[derive(Debug)]
pub enum AppError {
    Connection(fchat::Error),
}

enum AppScreen {
    Login {
        focus: u8,
        username: TextState<'static>,
        password: TextState<'static>,
    },
    Characters {
        ticket: Ticket,
        list_state: ListState,
    },
    Chat {
        ticket: Ticket,
        text_state: TextAreaState,
    },
}
