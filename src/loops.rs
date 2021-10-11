// jkcoxson

use std::sync::Arc;

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt,
};
use serde_json::Value;
use tokio::{
    net::TcpStream,
    sync::{
        mpsc::{UnboundedReceiver, UnboundedSender},
        Mutex,
    },
    time::{self, Duration, Instant},
};
use tokio_stream::StreamExt;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::{client::ClientData, opcodes};

pub struct Channels {
    pub reader_break_sender: Option<UnboundedSender<u8>>,
    pub writer_break_sender: Option<UnboundedSender<u8>>,
    pub heartbeat_break_sender: Option<UnboundedSender<u8>>,
}

impl Channels {
    pub fn new() -> Self {
        return Channels {
            reader_break_sender: None,
            writer_break_sender: None,
            heartbeat_break_sender: None,
        };
    }
    pub async fn reader(
        mut receiver: UnboundedReceiver<u8>,
        arc_reactor: Arc<Mutex<ClientData>>,
        mut reader: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    ) {
        loop {
            tokio::select! {
                _ = receiver.recv() => {
                    println!("Received a stop request");
                    break;
                }
                msg = reader.next() => {
                    match msg {
                        Some(msg) => match msg {
                            Ok(msg) => {
                                handle_packet(arc_reactor.clone(), msg.to_string()).await;
                            }
                            Err(err) => {println!("{}", err)},
                        },
                        None => {
                            println!("Stream exhausted");
                            break;
                        },
                    }

                }
            }
        }
        println!("Reader loop broken");
    }
    pub async fn writer(
        mut receiver: UnboundedReceiver<u8>,
        mut write_receiver: UnboundedReceiver<String>,
        mut writer: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    ) {
        loop {
            tokio::select! {
                _ = receiver.recv() => {
                    break;
                }
                rcvd = write_receiver.recv() => {
                    match rcvd {
                        Some(msg) => {
                        match writer.send(Message::Text(msg.to_owned())).await {
                            _ => {
                                // yay error handling
                                // I don't care
                            }
                        }
                    }
                    None => {}
                    }
                }
            }
        }
    }
    pub async fn heartbeat(
        mut receiver: UnboundedReceiver<u8>,
        arc_reactor: Arc<Mutex<ClientData>>,
        sender: UnboundedSender<String>,
    ) {
        let mut interval = 5000;
        loop {
            let sleep = time::sleep(Duration::from_millis(interval));
            tokio::pin!(sleep);
            tokio::select! {
                _ = receiver.recv() => {
                    break;
                },
                () = &mut sleep => {
                    if interval == 5000 {
                        let guard = arc_reactor.lock().await;
                        match guard.heartbeat_interval {
                            Some(guard_int) => interval = guard_int,
                            None => {}
                        }
                        std::mem::drop(guard);
                    } else {
                        // TODO do something with d, idk what yet
                        match sender.send(serde_json::to_string(&opcodes::OP1::new(0)).unwrap()) {
                            _ => {
                                // useless output
                            }
                        }
                    }
                    sleep.as_mut().reset(Instant::now() + Duration::from_millis(interval));
                }
            }
        }
        println!("Heartbeat loop broken");
    }
}

async fn handle_packet(client_data: Arc<Mutex<ClientData>>, raw_packet: String) {
    println!("{}", raw_packet);
    let mut guard = client_data.lock().await;
    let v: Option<Value> = serde_json::from_str(&raw_packet).unwrap_or(None);
    match v {
        Some(rawpkt) => {
            match &rawpkt["op"] {
                Value::Number(opcode) => {
                    println!("opcode: {}", opcode);
                    match opcode.as_i64().unwrap() {
                        0 => {
                            // let callbacks = guard.callbacks.clone();
                            // drop(guard);
                            // callbacks.iter().for_each(|function| {
                            //     function.run();
                            // })
                        }
                        7 => {
                            todo!("Reconnect when recieving a request to do so");
                        }
                        9 => {
                            panic!("Invalid session");
                        }
                        10 => {
                            println!("Hello packet");
                            println!("Heartbeat Packet: {}", &rawpkt["d"]["heartbeat_interval"]);
                            guard.heartbeat_interval = rawpkt["d"]["heartbeat_interval"].as_u64();
                        }
                        11 => {
                            println!("Heartbeat ACK");
                        }
                        _ => {
                            println!("Unknown opcode");
                        }
                    }
                }
                _ => {}
            }
            // Handle S values for reconnects
            let mut s_guard = client_data.lock().await;
            match &rawpkt["s"] {
                Value::Number(s) => s_guard.last_packet_id = s.as_u64().unwrap(),
                _ => {}
            }
        }
        None => return,
    }
}
