// Represents a Discord client
// https://discord.js.org/#/docs/main/stable/class/Client
// Based from this ^^

use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url;

use crate::opcodes;

pub struct Client {
    unbounded_receiver: UnboundedReceiver<String>,
    unbounded_sender: UnboundedSender<String>,
    client_data: Arc<Mutex<ClientData>>,
}

impl Client {
    /// Creates a new Discord client
    /// TODO: have intent arguments here
    pub fn new() -> Self {
        let (us, ur) = tokio::sync::mpsc::unbounded_channel::<String>();
        let arc_reactor = Arc::new(Mutex::new(ClientData::new()));
        return Client {
            unbounded_receiver: ur,
            unbounded_sender: us,
            client_data: arc_reactor,
        };
    }

    /// Creates a connection to Discord and starts
    pub async fn login(self, token: &str) {
        println!("Connecting to Discord gateway");

        let url = url::Url::parse("wss://gateway.discord.gg/?v=9&encoding=json").unwrap();
        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
        let (mut writer, mut reader) = ws_stream.split();
        let mut uxr = self.unbounded_receiver;

        // Spawn the WebSocket loop handler
        tokio::task::spawn(async move {
            loop {
                tokio::select! {
                    msg = reader.next() => {
                        match msg {
                            Some(msg) => {
                                match msg {
                                    Ok(msg) => {
                                        println!("{}", msg);
                                    },
                                    Err(_) => {
                                        // errors idk
                                    }
                                }
                            }
                            None => {
                                // We've disconnected, handle that pls
                                break;
                            }
                        }
                    }
                    msg = uxr.recv() => {
                        match writer.send(Message::Text(msg.to_owned().unwrap())).await {
                            _ => {
                                // yay error handling
                                // I don't care
                            }
                        }
                    }
                }
            }
        });

        // Heartbeat packet loop
        let usx = self.unbounded_sender.clone();
        let arc_reactor = self.client_data.clone();
        tokio::task::spawn(async move {
            let mut intverval: Option<u64> = None;
            loop {
                if intverval == None {
                    let guard = arc_reactor.lock().await;
                    match guard.heartbeat_interval {
                        Some(guard_int) => intverval = Some(guard_int),
                        None => {}
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                } else {
                    // TODO do something with d, idk what yet
                    match usx.send(serde_json::to_string(&opcodes::OP1::new(0)).unwrap()) {
                        _ => {
                            // useless output
                        }
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(intverval.unwrap()))
                        .await;
                }
            }
        });

        // Send identification packet
        let tkn = token.clone().to_string();
        let usx = self.unbounded_sender.clone();
        tokio::task::spawn(async move {
            match usx.send(
                serde_json::to_string(&opcodes::OP2::new(tkn.to_string(), 7))
                    .unwrap()
                    .replace("\\", "")
                    .replace("\"{", "{")
                    .replace("}\"", "}"),
            ) {
                _ => {
                    // lol
                }
            }
        });
    }
}

pub struct ClientData {
    heartbeat_interval: Option<u64>,
}

impl ClientData {
    fn new() -> Self {
        ClientData {
            heartbeat_interval: None,
        }
    }
}
