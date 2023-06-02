use std::sync::{Arc, Mutex};

use futures::{channel::mpsc::Sender, SinkExt, StreamExt};
use lib::message::{ClientMessage, ServerMessage};
use reqwasm::websocket::{futures::WebSocket, Message};

use wasm_bindgen_futures::spawn_local;
use yew_agent::Dispatched;

use self::event_bus::{EventBus, EventBusMessage};
pub mod event_bus;

pub struct Client {
    tx: Arc<Mutex<Option<Sender<ClientMessage>>>>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            tx: Arc::new(Mutex::new(None)),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.tx.lock().unwrap().is_some()
    }

    pub fn connect(&self) -> bool {
        let ws = WebSocket::open("ws:[::1]:8080/websocket").unwrap();

        let (mut write, mut read) = ws.split();

        let (in_tx, mut in_rx) = futures::channel::mpsc::channel::<ClientMessage>(1000);
        let mut event_bus = EventBus::dispatcher();

        spawn_local(async move {
            while let Some(message) = in_rx.next().await {
                log::debug!("send message to server");
                let s = serde_json::to_string(&message).unwrap();
                write.send(Message::Text(s)).await.unwrap();
            }
        });

        spawn_local(async move {
            while let Some(message) = read.next().await {
                match message {
                    Ok(Message::Text(message)) => {
                        log::debug!("received message from server");
                        let message: ServerMessage = serde_json::from_str(&message).unwrap();
                        event_bus.send(EventBusMessage { message });
                    }
                    Ok(Message::Bytes(b)) => {
                        let decoded = std::str::from_utf8(&b);
                        if let Ok(_val) = decoded {
                            //TODO:
                        }
                    }
                    Err(_e) => {}
                }
            }
        });

        *self.tx.lock().unwrap() = Some(in_tx);
        true
    }

    pub fn send_message_to_server(&self, message: ClientMessage) -> bool {
        let mut sender_lock = self.tx.lock().unwrap();
        if sender_lock.is_none() {
            log::debug!("sender is none");
            return false;
        }

        let mut sender = sender_lock.take().unwrap();
        let _ = sender.try_send(message);
        log::debug!("sender sent message");

        *sender_lock = Some(sender);

        true
    }
}