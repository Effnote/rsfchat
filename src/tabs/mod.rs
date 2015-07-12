use std::borrow::ToOwned;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use message::{ServerMessage, ClientMessage, out};
use config::{self, Config};
use WsSender;

use self::dropping_buf::DroppingBuf;
use self::messages::{Message, MessageType, Log};

mod dropping_buf;
pub mod messages;

struct StatusTab {
    messages: Log,
}

struct ChannelTab {
    joined: bool,
    name: String,
    id: Rc<String>,
    messages: Log,
}

struct PrivateTab {
    id: Rc<String>,
    messages: Log,
}

pub trait Tab {
    fn send_message(&mut self, sender: &mut WsSender, kind: MessageType, message: &str) -> Result<(), &'static str>;
    fn get_mut_log(&mut self) -> &mut Log;
    fn get_log(&self) -> &Log;
    fn add_message(&mut self, kind: MessageType, message: &str) {
        self.get_mut_log().add_message(kind, message);
    }
}

impl Tab for StatusTab {
    fn send_message(&mut self, sender: &mut WsSender, kind: MessageType, message: &str) -> Result<(), &'static str> {
        Err("Can't send a message into the status window.")
    }

    fn get_mut_log(&mut self) -> &mut Log {
        &mut self.messages
    }

    fn get_log(&self) -> &Log {
        &self.messages
    }
}

impl Tab for ChannelTab {
    fn send_message(&mut self, sender: &mut WsSender, kind: MessageType, message: &str) -> Result<(), &'static str> {
        out::MSG {
            channel: &**self.id,
            message: message
        }.send(sender);
        self.add_message(kind, message);
        Ok(())
    }

    fn get_mut_log(&mut self) -> &mut Log {
        &mut self.messages
    }

    fn get_log(&self) -> &Log {
        &self.messages
    }
}

impl Tab for PrivateTab {
    fn send_message(&mut self, sender: &mut WsSender, kind: MessageType, message: &str) -> Result<(), &'static str> {
        out::PRI {
            recipient: &**self.id,
            message: message
        }.send(sender);
        self.add_message(kind, message);
        Ok(())
    }

    fn get_mut_log(&mut self) -> &mut Log {
        &mut self.messages
    }

    fn get_log(&self) -> &Log {
        &self.messages
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum TabType<T> {
    Status,
    Channel(T),
    Private(T),
}

pub struct Tabs {
    status: StatusTab,
    channel_tabs: HashMap<Rc<String>, ChannelTab>,
    private_tabs: HashMap<Rc<String>, PrivateTab>,
    tab_order: Vec<TabType<Rc<String>>>,
    closed_channel_tabs: HashMap<Rc<String>, ChannelTab>,
    closed_private_tabs: HashMap<Rc<String>, PrivateTab>,
    current_tab: u16,
    joined_count: u16,
    display_buffer_size: u16,
    settings: config::Settings,
    formatter: Box<Fn() -> Box<Fn(&Message) -> String + 'static> + 'static>,
}

impl Tabs {
    // Closure returning a closure hack, to work around the lack of copyable closures.
    pub fn new<T: Fn() -> Box<Fn(&Message) -> String + 'static> + 'static>(config: &Config, formatter: T) -> Tabs {
        Tabs {
            status: StatusTab {
                messages: Log::new(config.settings.buffer_size, formatter()),
            },
            channel_tabs: HashMap::new(),
            private_tabs: HashMap::new(),
            tab_order: vec![],
            closed_channel_tabs: HashMap::new(),
            closed_private_tabs: HashMap::new(),
            current_tab: 0,
            joined_count: 0,
            display_buffer_size: config.settings.buffer_size,
            settings: config.settings,
            formatter: Box::new(formatter),
        }
    }

    pub fn get_current(&mut self) -> Result<&mut Tab, &'static str> {
        use self::TabType::*;
        let Tabs { ref mut status, ref mut channel_tabs, ref mut private_tabs, ref mut current_tab, ref tab_order, .. } = *self;
        if *current_tab == 0 {
            Ok(status)
        } else {
            if let Some(tab_type) = tab_order.get(*current_tab as usize - 1) {
                Ok(match *tab_type {
                    Status => status,
                    Channel(ref id) => match channel_tabs.get_mut(id) {
                        Some(tab) => tab as &mut Tab,
                        None => return Err("Couldn't find tab in channel_tabs"),
                    },
                    Private(ref id) => match private_tabs.get_mut(id) {
                        Some(tab) => tab as &mut Tab,
                        None => return Err("Couldn't find tab in private_tabs")
                    },
                })
            } else {
                // current_tab wasn't a valid index, probably should log it.
                *current_tab = 0;
                Err("current_tab wasn't valid")
            }
        }
    }

    pub fn all_joined(&self) -> bool {
        self.tab_order.len() == self.joined_count as usize
    }

    pub fn add_tab(&mut self, id: &str, private: bool) {
        use self::TabType::*;
        let id = Rc::new(id.to_owned());
        let kind;
        if private {
            kind = Private(id.clone());
            self.joined_count += 1;
            if let Some(tab) = self.closed_private_tabs.remove(&*id) {
                self.private_tabs.insert(id, tab);
            } else {
                let tab = PrivateTab {
                    id: id.clone(),
                    messages: Log::new(self.settings.buffer_size, (self.formatter)()),
                };
                self.private_tabs.insert(id, tab);
            }
        } else {
            kind = Channel(id.to_owned());
            if let Some(tab) = self.closed_channel_tabs.remove(&*id) {
                self.channel_tabs.insert(id, tab);
            } else {
                let tab = ChannelTab {
                    joined: false,
                    name: (&*id).to_owned(),
                    id: id.clone(),
                    messages: Log::new(self.settings.buffer_size, (self.formatter)()),
                };
                self.channel_tabs.insert(id, tab);
            }
        };
        self.tab_order.push(kind);
        self.current_tab = self.tab_order.len() as u16;
    }

    pub fn dispatch(&mut self, msg: ServerMessage) {
        use message::ServerMessage::*;
        /*
        match msg {
        }
        */
    }
}
