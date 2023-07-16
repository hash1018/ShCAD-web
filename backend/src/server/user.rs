use std::{fmt, sync::Arc};

use axum::extract::ws::{Message, WebSocket};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use lib::message::{ClientMessage, ServerMessage};
use tokio::sync::{mpsc::Sender, Mutex};
use tracing::log::{self};

use super::room::RoomMessage;

pub struct User {
    id: Arc<str>,
    room_sender: Arc<Mutex<Option<Sender<RoomMessage>>>>,
    socket_sender: SplitSink<WebSocket, Message>,
    socket_receiver: Option<SplitStream<WebSocket>>,
}

impl fmt::Debug for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("User").field("id", &self.id).finish()
    }
}

impl User {
    pub fn new(
        id: Arc<str>,
        sender: SplitSink<WebSocket, Message>,
        receiver: SplitStream<WebSocket>,
    ) -> Self {
        Self {
            id,
            room_sender: Arc::new(Mutex::new(None)),
            socket_sender: sender,
            socket_receiver: Some(receiver),
        }
    }

    pub async fn set_channel(&mut self, room_sender: Sender<RoomMessage>) {
        let id = self.id.clone();

        *self.room_sender.lock().await = Some(room_sender.clone());
        let room_sender_clone = self.room_sender.clone();

        let socket_receiver = self.socket_receiver.take().unwrap();

        tokio::spawn(async move {
            handle_message(id, room_sender_clone, socket_receiver).await;
        });
    }

    pub fn id(&self) -> Arc<str> {
        self.id.clone()
    }

    pub async fn send_message(&mut self, message: ServerMessage) {
        let message = serde_json::to_string(&message).unwrap();
        let _ = self.socket_sender.send(Message::Text(message)).await;
    }
}

async fn handle_message(
    id: Arc<str>,
    room_sender: Arc<Mutex<Option<Sender<RoomMessage>>>>,
    mut socket_receiver: SplitStream<WebSocket>,
) {
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(message)) = socket_receiver.next().await {
            if let Message::Text(message) = message {
                let message: ClientMessage = serde_json::from_str(&message).unwrap();
                let room_message = match message {
                    ClientMessage::Leave => {
                        let sender_lock = room_sender.lock().await;
                        if let Some(sender) = &*sender_lock {
                            sender.send(RoomMessage::LeaveUser(id)).await.unwrap();
                        }
                        break;
                    }
                    ClientMessage::AddFigure(data) => RoomMessage::AddFigure(data),
                    ClientMessage::RequestInfo(request_type) => {
                        RoomMessage::RequestInfo(id.clone(), request_type)
                    }
                    ClientMessage::NotifyMousePositionChanged(queue) => {
                        RoomMessage::NotifyMousePositionChanged(id.clone(), queue)
                    }
                    _ => {
                        continue;
                    }
                };

                let sender_lock = room_sender.lock().await;
                if let Some(sender) = &*sender_lock {
                    sender.send(room_message).await.unwrap();
                }
            } else if let Message::Close(_) = message {
                let sender_lock = room_sender.lock().await;
                if let Some(sender) = &*sender_lock {
                    sender.send(RoomMessage::LeaveUser(id)).await.unwrap();
                }
                break;
            } else {
                log::info!("other message {message:?}");
            }
        }
    });

    tokio::select! {
        _ = (&mut recv_task) => {}
    };
}
