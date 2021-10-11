// jkcoxson
// Represents a message

use crate::{channel::Channel, guild_member::GuildMember};

pub struct Message {
    content: String,
    // author: GuildMember,
    // channel: Channel,
    id: u64,
}

impl Message {
    // pub fn new() -> Self {
    //     Message {}
    // }
}
