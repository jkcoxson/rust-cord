// Represents a Discord client
// https://discord.js.org/#/docs/main/stable/class/Client
// Based from this ^^

use futures_util::StreamExt;
use std::mem::drop;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::Mutex;
use tokio_tungstenite::connect_async;
use url;

use crate::callbacks::{self, Callbacks};
use crate::{loops, message::Message, opcodes};

pub struct Client {
    client_data: Arc<Mutex<ClientData>>,
}

pub struct ClientData {
    pub heartbeat_interval: Option<u64>,
    pub callbacks: Callbacks,
    loops: loops::Channels,
    pub last_packet_id: u64,
}

impl ClientData {
    fn new() -> Self {
        ClientData {
            heartbeat_interval: None,
            callbacks: Callbacks::new(),
            loops: loops::Channels::new(),
            last_packet_id: 0,
        }
    }
}

impl Client {
    /// Creates a new Discord client
    /// TODO: have intent arguments here
    pub fn new() -> Self {
        let arc_reactor = Arc::new(Mutex::new(ClientData::new()));
        return Client {
            client_data: arc_reactor,
        };
    }

    /// Creates a connection to Discord and starts
    pub async fn login(&self, token: &str) {
        println!("Connecting to Discord gateway");

        let url = url::Url::parse("wss://gateway.discord.gg/?v=9&encoding=json").unwrap();
        let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
        let (writer, reader) = ws_stream.split();
        let (usx, uxr) = tokio::sync::mpsc::unbounded_channel::<String>();

        // Values to move
        let arc_reactor = self.client_data.clone();
        let moved_usx = usx.clone();
        let heartbeat_usx = usx.clone();

        // Reader loop
        let (break_usx, break_uxr) = tokio::sync::mpsc::unbounded_channel::<u8>();
        let temp_reactor = self.client_data.clone();
        let mut guard = temp_reactor.lock().await;
        guard.loops.reader_break_sender = Some(break_usx);
        drop(guard);
        tokio::task::spawn(
            async move { loops::Channels::reader(break_uxr, arc_reactor, reader).await },
        );

        // Writer loop
        let (break_usx, break_uxr) = tokio::sync::mpsc::unbounded_channel::<u8>();
        let temp_reactor = self.client_data.clone();
        let mut guard = temp_reactor.lock().await;
        guard.loops.writer_break_sender = Some(break_usx);
        drop(guard);
        tokio::task::spawn(async move {
            loops::Channels::writer(break_uxr, uxr, writer).await;
        });

        // Heartbeat packet loop
        let (break_usx, break_uxr) = tokio::sync::mpsc::unbounded_channel::<u8>();
        let temp_reactor = self.client_data.clone();
        let mut guard = temp_reactor.lock().await;
        guard.loops.heartbeat_break_sender = Some(break_usx);
        drop(guard);
        let arc_reactor = self.client_data.clone();
        tokio::task::spawn(async move {
            loops::Channels::heartbeat(break_uxr, arc_reactor, heartbeat_usx).await
        });

        // Send identification packet
        let tkn = token.clone().to_string();
        tokio::task::spawn(async move {
            match moved_usx.send(
                serde_json::to_string(&opcodes::OP2::new(tkn.to_string(), 32767))
                    .unwrap()
                    .replace("\\", "")
                    .replace("\"{", "{")
                    .replace("}\"", "}"),
            ) {
                Ok(_) => {}
                Err(_) => panic!("Error sending identification packet"),
            }
        });
    }

    // pub async fn disconnect(&self) {
    //     let mut guard = self.client_data.lock().await;
    //     guard.loops.heartbeat_break_sender.as_ref().unwrap().send(0);
    //     guard.loops.reader_break_sender.as_ref().unwrap().send(0);
    //     guard.loops.writer_break_sender.as_ref().unwrap().send(0);
    // }

    /// Adds a function callback that will always fire when an event goes off
    pub async fn on_message_create(&self, id: u16) -> Option<Message> {
        let mut guard = self.client_data.lock().await;
        let (usx, mut usr) = unbounded_channel::<Message>();
        guard
            .callbacks
            .message_create
            .push(callbacks::MessageCreateCallback::new(usx, id, false));
        usr.recv().await
    }
}
