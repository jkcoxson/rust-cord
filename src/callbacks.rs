use tokio::sync::mpsc::UnboundedSender;

use crate::message::Message;

// jkcoxson

pub struct Callbacks {
    pub message_create: Vec<MessageCreateCallback>,
}

impl Callbacks {
    pub fn new() -> Self {
        Callbacks {
            message_create: vec![],
        }
    }
}

pub struct MessageCreateCallback {
    sender: UnboundedSender<Message>,
    id: u16,
    remove: bool,
}

impl MessageCreateCallback {
    pub fn new(sender: UnboundedSender<Message>, id: u16, remove: bool) -> Self {
        MessageCreateCallback { sender, id, remove }
    }
}
