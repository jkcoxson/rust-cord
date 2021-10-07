// Represents serializable opcodes

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OP1 {
    op: u8,
    d: u16,
}

impl OP1 {
    pub fn new(d: u16) -> Self {
        OP1 { op: 1, d }
    }
}

#[derive(Serialize, Deserialize)]
pub struct OP2 {
    op: u8,
    d: IdentityData,
}

#[derive(Serialize, Deserialize)]
pub struct IdentityData {
    token: String,
    intents: u16,
    properties: String,
}

impl OP2 {
    pub fn new(token: String, intents: u16) -> Self {
        OP2 {
            op: 2,
            d: IdentityData::new(token, intents, std::env::consts::OS.to_string()),
        }
    }
}

impl IdentityData {
    pub fn new(token: String, intents: u16, os: String) -> Self {
        IdentityData {
            token,
            intents,
            properties: format!(
                "{{\"$os\":\"{}\",\"$browser\":\"{}\",\"$device\":\"{}\"}}",
                os, "rust_cord", "rust_cord"
            ),
        }
    }
}
