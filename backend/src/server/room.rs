use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use lib::{
    figure::FigureData,
    message::{AcceptedType, NotifyType, RequestType, ResponseType, ServerMessage},
};
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    Mutex, MutexGuard,
};
use tracing::log;

use crate::syncing_system::{
    deletion::delete,
    selection::{select, unselect},
};

use super::{user::User, ServerAppMessage};

#[derive(Debug)]
pub enum RoomMessage {
    LeaveUser(Arc<str>),
    AddFigure(FigureData),
    RequestInfo(Arc<str>, RequestType),
    NotifyMousePositionChanged(Arc<str>, VecDeque<(f64, f64)>),
    SelectFigure(Arc<str>, BTreeSet<usize>),
    UnselectFigureAll(Arc<str>),
    NotifySelectDragStart(Arc<str>, f64, f64),
    NotifySelectDragFinish(Arc<str>),
    UpdateSelectedFigures(Arc<str>, Option<BTreeSet<usize>>, Option<BTreeSet<usize>>),
    DeleteFigures(Arc<str>, BTreeSet<usize>),
}

#[allow(clippy::type_complexity)]
pub struct Room {
    id: Arc<str>,
    server_app_sender: Sender<ServerAppMessage>,
    sender: Sender<RoomMessage>, //Pass to new_user so that room's receiver can receive a message from user.
    room_inner: Arc<Mutex<RoomInner>>,
}

impl Room {
    pub fn new(id: Arc<str>, server_app_sender: Sender<ServerAppMessage>) -> Self {
        let (sender, receiver) = mpsc::channel(1000);

        let room = Self {
            id,
            server_app_sender,
            sender,
            room_inner: Arc::new(Mutex::new(RoomInner::new())),
        };

        room.run(receiver);

        room
    }

    #[allow(clippy::single_match)]
    fn run(&self, mut receiver: Receiver<RoomMessage>) {
        let server_app_sender_clone = self.server_app_sender.clone();
        let room_id = self.id.clone();
        let room_inner = self.room_inner.clone();
        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                match message {
                    RoomMessage::LeaveUser(user_id) => {
                        log::info!("LeaveUser user_id = {user_id}");
                        let mut room_inner_lock = room_inner.lock().await;
                        room_inner_lock.users.remove(&user_id);
                        log::info!("now users = {0:?}", room_inner_lock.users);
                        if room_inner_lock.users.is_empty() {
                            let _ = server_app_sender_clone
                                .send(ServerAppMessage::DeleteRoom(room_id.clone()))
                                .await;
                            break;
                        } else {
                            broadcast(
                                &mut room_inner_lock.users,
                                ServerMessage::Notify(NotifyType::UserLeft(user_id.to_string())),
                            )
                            .await;
                        }

                        unselect_all(user_id, &mut room_inner_lock).await;
                    }
                    RoomMessage::AddFigure(data) => {
                        static FIGURE_ID: AtomicUsize = AtomicUsize::new(1);
                        let new_id = FIGURE_ID.fetch_add(1, Ordering::Relaxed);

                        let mut room_inner_lock = room_inner.lock().await;

                        room_inner_lock.figures.insert(new_id, data.clone());

                        broadcast(
                            &mut room_inner_lock.users,
                            ServerMessage::Notify(NotifyType::FigureAdded(new_id, data)),
                        )
                        .await;
                    }
                    RoomMessage::RequestInfo(user_id, request_type) => match request_type {
                        RequestType::CurrentFigures => {
                            let mut room_inner_lock = room_inner.lock().await;

                            let figures = room_inner_lock.figures.clone();

                            unicast(
                                &mut room_inner_lock.users,
                                &user_id,
                                ServerMessage::Response(ResponseType::CurrentFigures(figures)),
                            )
                            .await;
                        }
                        RequestType::CurrentSharedUsers => {
                            let mut room_inner_lock = room_inner.lock().await;

                            let mut vec = Vec::new();

                            for (user_id, _) in room_inner_lock.users.iter() {
                                vec.push(user_id.to_string());
                            }

                            unicast(
                                &mut room_inner_lock.users,
                                &user_id,
                                ServerMessage::Response(ResponseType::CurrentSharedUsers(vec)),
                            )
                            .await;
                        }
                        RequestType::CurrentSelectedFigures => {
                            let mut room_inner_lock = room_inner.lock().await;

                            let mut map = BTreeMap::new();
                            for (id, ids) in room_inner_lock.selected_figures.iter() {
                                map.insert(id.to_string(), ids.clone());
                            }

                            unicast(
                                &mut room_inner_lock.users,
                                &user_id,
                                ServerMessage::Response(ResponseType::CurrentSelectedFigures(map)),
                            )
                            .await;
                        }
                        RequestType::CurrentSelectDragPositions => {
                            let mut room_inner_lock = room_inner.lock().await;

                            let mut map = BTreeMap::new();
                            for (id, (x, y)) in room_inner_lock.select_drag_positions.iter() {
                                map.insert(id.to_string(), (*x, *y));
                            }

                            unicast(
                                &mut room_inner_lock.users,
                                &user_id,
                                ServerMessage::Response(ResponseType::CurrentSelectDragPositions(
                                    map,
                                )),
                            )
                            .await;
                        }
                        RequestType::CheckRoomExist(_) => {
                            unreachable!()
                        }
                        RequestType::CheckUserExist(_, _) => {
                            unreachable!()
                        }
                    },
                    RoomMessage::NotifyMousePositionChanged(user_id, queue) => {
                        let mut room_inner_lock = room_inner.lock().await;

                        broadcast_except_for(
                            &mut room_inner_lock.users,
                            &user_id,
                            ServerMessage::Notify(NotifyType::UserMousePositionChanged(
                                user_id.to_string(),
                                queue,
                            )),
                        )
                        .await;
                    }
                    RoomMessage::SelectFigure(user_id, ids) => {
                        let mut room_inner_lock = room_inner.lock().await;

                        let (accepted_set, _rejected_set) =
                            select(&mut room_inner_lock, &user_id, ids);

                        broadcast_except_for(
                            &mut room_inner_lock.users,
                            &user_id,
                            ServerMessage::Notify(NotifyType::FigureSelected(
                                user_id.to_string(),
                                accepted_set.clone(),
                            )),
                        )
                        .await;

                        unicast(
                            &mut room_inner_lock.users,
                            &user_id,
                            ServerMessage::Accepted(AcceptedType::FigureSelected(accepted_set)),
                        )
                        .await;
                    }
                    RoomMessage::UnselectFigureAll(user_id) => {
                        let mut room_inner_lock = room_inner.lock().await;
                        unselect_all(user_id, &mut room_inner_lock).await;
                    }
                    RoomMessage::NotifySelectDragStart(user_id, x, y) => {
                        let mut room_inner_lock = room_inner.lock().await;

                        room_inner_lock
                            .select_drag_positions
                            .insert(user_id.clone(), (x, y));

                        broadcast_except_for(
                            &mut room_inner_lock.users,
                            &user_id,
                            ServerMessage::Notify(NotifyType::SelectDragStarted(
                                user_id.to_string(),
                                x,
                                y,
                            )),
                        )
                        .await;
                    }
                    RoomMessage::NotifySelectDragFinish(user_id) => {
                        let mut room_inner_lock = room_inner.lock().await;

                        room_inner_lock.select_drag_positions.remove(&user_id);

                        broadcast_except_for(
                            &mut room_inner_lock.users,
                            &user_id,
                            ServerMessage::Notify(NotifyType::SelectDragFinished(
                                user_id.to_string(),
                            )),
                        )
                        .await;
                    }
                    RoomMessage::UpdateSelectedFigures(
                        user_id,
                        about_to_select_set,
                        about_to_unselect_set,
                    ) => {
                        let mut room_inner_lock = room_inner.lock().await;

                        let (accepted_select_set, _rejected_select_set) =
                            if let Some(about_to_select_set) = about_to_select_set {
                                let (a, r) =
                                    select(&mut room_inner_lock, &user_id, about_to_select_set);
                                (Some(a), Some(r))
                            } else {
                                (None, None)
                            };

                        let (accepted_unselect_set, _rejected_unselect_set) =
                            if let Some(about_to_unselect_set) = about_to_unselect_set {
                                let (a, r) =
                                    unselect(&mut room_inner_lock, &user_id, about_to_unselect_set);
                                (Some(a), Some(r))
                            } else {
                                (None, None)
                            };

                        broadcast_except_for(
                            &mut room_inner_lock.users,
                            &user_id,
                            ServerMessage::Notify(NotifyType::SelectedFiguresUpdated(
                                user_id.to_string(),
                                accepted_select_set.clone(),
                                accepted_unselect_set.clone(),
                            )),
                        )
                        .await;

                        unicast(
                            &mut room_inner_lock.users,
                            &user_id,
                            ServerMessage::Accepted(AcceptedType::SelectedFiguresUpdated(
                                accepted_select_set,
                                accepted_unselect_set,
                            )),
                        )
                        .await;
                    }
                    RoomMessage::DeleteFigures(user_id, ids) => {
                        let mut room_inner_lock = room_inner.lock().await;

                        let (accpeted_set, _rejected_set) = delete(&mut room_inner_lock, ids);

                        broadcast_except_for(
                            &mut room_inner_lock.users,
                            &user_id,
                            ServerMessage::Notify(NotifyType::FigureDeleted(accpeted_set.clone())),
                        )
                        .await;

                        unicast(
                            &mut room_inner_lock.users,
                            &user_id,
                            ServerMessage::Accepted(AcceptedType::FigureDeleted(accpeted_set)),
                        )
                        .await;
                    }
                }
            }
        });
    }

    pub async fn join_user(&self, mut new_user: User) {
        let mut room_inner_lock = self.room_inner.lock().await;
        let new_user_id = new_user.id();
        new_user.set_channel(self.sender.clone()).await;

        room_inner_lock.users.insert(new_user.id(), new_user);

        broadcast_except_for(
            &mut room_inner_lock.users,
            &new_user_id,
            ServerMessage::Notify(NotifyType::UserJoined(new_user_id.to_string())),
        )
        .await;
        unicast(
            &mut room_inner_lock.users,
            &new_user_id,
            ServerMessage::Accepted(AcceptedType::UserJoined),
        )
        .await;
    }

    pub async fn check_exist_user(&self, user_id: &str) -> bool {
        self.room_inner.lock().await.users.get(user_id).is_some()
    }
}

pub struct RoomInner {
    pub users: HashMap<Arc<str>, User>,
    pub figures: BTreeMap<usize, FigureData>,
    pub selected_figures: BTreeMap<Arc<str>, BTreeSet<usize>>,
    pub select_drag_positions: BTreeMap<Arc<str>, (f64, f64)>,
}

impl RoomInner {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            figures: BTreeMap::new(),
            selected_figures: BTreeMap::new(),
            select_drag_positions: BTreeMap::new(),
        }
    }
}

async fn broadcast(users_lock: &mut HashMap<Arc<str>, User>, message: ServerMessage) {
    for (_, user) in users_lock.iter_mut() {
        user.send_message(message.clone()).await;
    }
}

async fn broadcast_except_for(
    users_lock: &mut HashMap<Arc<str>, User>,
    except_user_id: &Arc<str>,
    message: ServerMessage,
) {
    for (id, user) in users_lock.iter_mut() {
        if id != except_user_id {
            user.send_message(message.clone()).await;
        }
    }
}

async fn unicast(
    users_lock: &mut HashMap<Arc<str>, User>,
    user_id: &Arc<str>,
    message: ServerMessage,
) {
    if let Some(user) = users_lock.get_mut(user_id) {
        user.send_message(message.clone()).await;
    }
}

async fn unselect_all(user_id: Arc<str>, room_inner_lock: &mut MutexGuard<'_, RoomInner>) {
    room_inner_lock.selected_figures.remove(&user_id);

    broadcast_except_for(
        &mut room_inner_lock.users,
        &user_id,
        ServerMessage::Notify(NotifyType::FigureUnselectedAll(user_id.to_string())),
    )
    .await;

    unicast(
        &mut room_inner_lock.users,
        &user_id,
        ServerMessage::Accepted(AcceptedType::FigureUnselectedAll),
    )
    .await;
}
