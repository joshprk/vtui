use std::sync::mpsc::Sender;

use ratatui::prelude::Backend;

use crate::events::Message;

pub trait Driver {
    type Backend: Backend;
}
