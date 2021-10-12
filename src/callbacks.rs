use tokio::sync::mpsc::UnboundedSender;

// jkcoxson

pub struct Callbacks {
    pub message_create: Vec<UnboundedSender<u64>>,
}

impl Callbacks {
    pub fn new() -> Self {
        Callbacks {
            message_create: vec![],
        }
    }
}
