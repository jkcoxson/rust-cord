// jkcoxson
// Represents a message

use serde_json::Value;

pub struct Message {
    pub content: String,
    // author: GuildMember,
    // channel: Channel,
    pub id: u64,
}

impl Message {
    pub fn new(d: Value) -> Self {
        let content = d["content"].as_str().unwrap().to_string();
        let id: u64 = d["id"].as_str().unwrap().parse::<u64>().unwrap();
        Message { content, id }
    }
}
