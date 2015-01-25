use std::borrow::ToOwned;

use message::{ClientMessage, out};
use WsSender;

pub struct Tab {
    pub joined: bool,
    pub name: String,
    pub kind: TabType,
    pub open: bool,
}

impl Tab {
    pub fn send_message(&self, sender: &mut WsSender, message: &str) -> Result<(), &'static str> {
        use self::TabType::*;
        match self.kind {
            Status => return Err("Can't send a message into the status window."),
            Channel(ref name) => {
                out::MSG {
                    channel: &**name,
                    message: message
                }.send(sender);
            }
            Private(ref name) => {
                out::PRI {
                    recipient: &**name,
                    message: message
                }.send(sender);
            }
        }
        Ok(())
    }
}

pub enum TabType {
    Status,
    Channel(String),
    Private(String),
}

pub struct Tabs {
    status: Tab,
    tabs: Vec<Tab>,
    current_tab: u16,
}

impl Tabs {
    pub fn new() -> Tabs {
        Tabs {
            status: Tab {
                joined: true,
                name: "Status".to_owned(),
                kind: TabType::Status,
                open: true,
            },
            tabs: vec![],
            current_tab: 0,
        }
    }

    pub fn get_current(&mut self) -> &Tab {
        let Tabs { ref status, ref tabs, ref mut current_tab, .. } = *self;
        if *current_tab == 0 {
            status
        } else {
            if let Some(tab) = tabs.get(*current_tab as usize - 1) {
                tab
            } else {
                *current_tab = 0;
                status
            }
        }
    }

    pub fn get(&self, index: u16) -> Option<&Tab> {
        if index == 0 {
            Some(&self.status)
        } else {
            self.tabs.get(index as usize - 1)
        }
    }
}
