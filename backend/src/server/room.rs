use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use lib::{
    figure::FigureData,
    message::{RequestType, ResponseType, ServerMessage},
};
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    Mutex, MutexGuard,
};
use tracing::log;

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
    users: Arc<Mutex<HashMap<Arc<str>, User>>>,
    figures: Arc<Mutex<BTreeMap<usize, FigureData>>>,
    selected_figures: Arc<Mutex<BTreeMap<Arc<str>, BTreeSet<usize>>>>,
    select_drag_positions: Arc<Mutex<BTreeMap<Arc<str>, (f64, f64)>>>,
    sender: Sender<RoomMessage>, //Pass to new_user so that room's receiver can receive a message from user.
}

impl Room {
    pub fn new(id: Arc<str>, server_app_sender: Sender<ServerAppMessage>) -> Self {
        let (sender, receiver) = mpsc::channel(1000);

        let room = Self {
            id,
            server_app_sender,
            users: Arc::new(Mutex::new(HashMap::new())),
            figures: Arc::new(Mutex::new(BTreeMap::new())),
            selected_figures: Arc::new(Mutex::new(BTreeMap::new())),
            select_drag_positions: Arc::new(Mutex::new(BTreeMap::new())),
            sender,
        };

        room.run(receiver);

        room
    }

    #[allow(clippy::single_match)]
    fn run(&self, mut receiver: Receiver<RoomMessage>) {
        let users_clone = self.users.clone();
        let server_app_sender_clone = self.server_app_sender.clone();
        let figures_clone = self.figures.clone();
        let selected_figures_clone = self.selected_figures.clone();
        let select_drag_positions_clone = self.select_drag_positions.clone();
        let room_id = self.id.clone();
        tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                match message {
                    RoomMessage::LeaveUser(user_id) => {
                        {
                            log::info!("LeaveUser user_id = {user_id}");
                            let mut users_lock = users_clone.lock().await;
                            users_lock.remove(&user_id);
                            log::info!("now users = {0:?}", *users_lock);
                            if users_lock.is_empty() {
                                let _ = server_app_sender_clone
                                    .send(ServerAppMessage::DeleteRoom(room_id.clone()))
                                    .await;
                                break;
                            } else {
                                broadcast(
                                    &mut users_lock,
                                    ServerMessage::UserLeft(user_id.to_string()),
                                )
                                .await;
                            }
                        }
                        {
                            unselect_all(
                                user_id,
                                selected_figures_clone.clone(),
                                users_clone.clone(),
                            )
                            .await;
                        }
                    }
                    RoomMessage::AddFigure(data) => {
                        static FIGURE_ID: AtomicUsize = AtomicUsize::new(1);
                        let new_id = FIGURE_ID.fetch_add(1, Ordering::Relaxed);

                        figures_clone.lock().await.insert(new_id, data.clone());
                        let mut users_lock = users_clone.lock().await;
                        broadcast(&mut users_lock, ServerMessage::FigureAdded(new_id, data)).await;
                    }
                    RoomMessage::RequestInfo(user_id, request_type) => match request_type {
                        RequestType::CurrentFigures => {
                            let mut users_lock = users_clone.lock().await;
                            let figures = figures_clone.lock().await.clone();
                            if let Some(user) = users_lock.get_mut(&user_id) {
                                user.send_message(ServerMessage::ResponseInfo(
                                    ResponseType::CurrentFigures(figures),
                                ))
                                .await;
                            }
                        }
                        RequestType::CurrentSharedUsers => {
                            let mut users_lock = users_clone.lock().await;
                            let mut vec = Vec::new();
                            for (user_id, _) in users_lock.iter() {
                                vec.push(user_id.to_string());
                            }

                            unicast(
                                &mut users_lock,
                                &user_id,
                                ServerMessage::ResponseInfo(ResponseType::CurrentSharedUsers(vec)),
                            )
                            .await;
                        }
                        RequestType::CurrentSelectedFigures => {
                            let mut users_lock = users_clone.lock().await;
                            let selected_figures_lock = selected_figures_clone.lock().await;

                            let mut map = BTreeMap::new();
                            for (id, ids) in selected_figures_lock.iter() {
                                map.insert(id.to_string(), ids.clone());
                            }

                            unicast(
                                &mut users_lock,
                                &user_id,
                                ServerMessage::ResponseInfo(ResponseType::CurrentSelectedFigures(
                                    map,
                                )),
                            )
                            .await;
                        }
                        RequestType::CurrentSelectDragPositions => {
                            let mut users_lock = users_clone.lock().await;
                            let select_drag_positions_lock =
                                select_drag_positions_clone.lock().await;

                            let mut map = BTreeMap::new();
                            for (id, (x, y)) in select_drag_positions_lock.iter() {
                                map.insert(id.to_string(), (*x, *y));
                            }

                            unicast(
                                &mut users_lock,
                                &user_id,
                                ServerMessage::ResponseInfo(
                                    ResponseType::CurrentSelectDragPositions(map),
                                ),
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
                        let mut users_lock = users_clone.lock().await;
                        broadcast_except_for(
                            &mut users_lock,
                            &user_id,
                            ServerMessage::NotifyUserMousePositionChanged(
                                user_id.to_string(),
                                queue,
                            ),
                        )
                        .await;
                    }
                    RoomMessage::SelectFigure(user_id, ids) => {
                        let mut selected_figures_lock = selected_figures_clone.lock().await;
                        let figures_lock = figures_clone.lock().await;

                        let (accepted_set, _rejected_set) =
                            select(&mut selected_figures_lock, &figures_lock, &user_id, ids);

                        let mut users_lock = users_clone.lock().await;
                        broadcast(
                            &mut users_lock,
                            ServerMessage::FigureSelected(user_id.to_string(), accepted_set),
                        )
                        .await;
                    }
                    RoomMessage::UnselectFigureAll(user_id) => {
                        unselect_all(user_id, selected_figures_clone.clone(), users_clone.clone())
                            .await;
                    }
                    RoomMessage::NotifySelectDragStart(user_id, x, y) => {
                        let mut select_drag_positions_lock =
                            select_drag_positions_clone.lock().await;
                        select_drag_positions_lock.insert(user_id.clone(), (x, y));

                        let mut users_lock = users_clone.lock().await;
                        broadcast_except_for(
                            &mut users_lock,
                            &user_id,
                            ServerMessage::NotifySelectDragStarted(user_id.to_string(), x, y),
                        )
                        .await;
                    }
                    RoomMessage::NotifySelectDragFinish(user_id) => {
                        let mut select_drag_positions_lock =
                            select_drag_positions_clone.lock().await;
                        select_drag_positions_lock.remove(&user_id);

                        let mut users_lock = users_clone.lock().await;
                        broadcast_except_for(
                            &mut users_lock,
                            &user_id,
                            ServerMessage::NotifySelectDragFinished(user_id.to_string()),
                        )
                        .await;
                    }
                    RoomMessage::UpdateSelectedFigures(
                        user_id,
                        about_to_select_set,
                        about_to_unselect_set,
                    ) => {
                        let mut selected_figures_lock = selected_figures_clone.lock().await;
                        let figures_lock = figures_clone.lock().await;
                        let (accepted_select_set, _rejected_select_set) =
                            if let Some(about_to_select_set) = about_to_select_set {
                                let (a, r) = select(
                                    &mut selected_figures_lock,
                                    &figures_lock,
                                    &user_id,
                                    about_to_select_set,
                                );
                                (Some(a), Some(r))
                            } else {
                                (None, None)
                            };

                        let (accepted_unselect_set, _rejected_unselect_set) =
                            if let Some(about_to_unselect_set) = about_to_unselect_set {
                                let (a, r) = unselect(
                                    &mut selected_figures_lock,
                                    &figures_lock,
                                    &user_id,
                                    about_to_unselect_set,
                                );
                                (Some(a), Some(r))
                            } else {
                                (None, None)
                            };

                        let mut users_lock = users_clone.lock().await;
                        broadcast(
                            &mut users_lock,
                            ServerMessage::SelectedFiguresUpdated(
                                user_id.to_string(),
                                accepted_select_set,
                                accepted_unselect_set,
                            ),
                        )
                        .await;
                    }
                    RoomMessage::DeleteFigures(user_id, ids) => {
                        let mut selected_figures_lock = selected_figures_clone.lock().await;
                        let mut remove_vec = Vec::new();
                        for (remove_id, set) in selected_figures_lock.iter_mut() {
                            for i in ids.iter() {
                                set.remove(i);
                            }
                            if set.is_empty() {
                                remove_vec.push(remove_id.clone());
                            }
                        }

                        for remove_id in remove_vec {
                            selected_figures_lock.remove(&remove_id);
                        }

                        let mut figures_lock = figures_clone.lock().await;
                        for id in ids.iter() {
                            figures_lock.remove(id);
                        }

                        let mut users_lock = users_clone.lock().await;
                        broadcast(
                            &mut users_lock,
                            ServerMessage::FigureDeleted(user_id.to_string(), ids),
                        )
                        .await;
                    }
                }
            }
        });
    }

    pub async fn join_user(&self, mut new_user: User) {
        let new_user_id = new_user.id();
        new_user.set_channel(self.sender.clone()).await;

        let mut users_lock = self.users.lock().await;
        users_lock.insert(new_user.id(), new_user);

        broadcast(
            &mut users_lock,
            ServerMessage::UserJoined(new_user_id.to_string()),
        )
        .await;
    }

    pub async fn check_exist_user(&self, user_id: &str) -> bool {
        self.users.lock().await.get(user_id).is_some()
    }
}

async fn broadcast(
    users_lock: &mut MutexGuard<'_, HashMap<Arc<str>, User>>,
    message: ServerMessage,
) {
    for (_, user) in users_lock.iter_mut() {
        user.send_message(message.clone()).await;
    }
}

async fn broadcast_except_for(
    users_lock: &mut MutexGuard<'_, HashMap<Arc<str>, User>>,
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
    users_lock: &mut MutexGuard<'_, HashMap<Arc<str>, User>>,
    user_id: &Arc<str>,
    message: ServerMessage,
) {
    if let Some(user) = users_lock.get_mut(user_id) {
        user.send_message(message.clone()).await;
    }
}

async fn unselect_all(
    user_id: Arc<str>,
    selected_figures: Arc<Mutex<BTreeMap<Arc<str>, BTreeSet<usize>>>>,
    users: Arc<Mutex<HashMap<Arc<str>, User>>>,
) {
    let mut selected_figures_lock = selected_figures.lock().await;
    selected_figures_lock.remove(&user_id);

    let mut users_lock = users.lock().await;
    broadcast(
        &mut users_lock,
        ServerMessage::FigureUnselectedAll(user_id.to_string()),
    )
    .await;
}

fn select(
    selected_figures_lock: &mut MutexGuard<'_, BTreeMap<Arc<str>, BTreeSet<usize>>>,
    figures_lock: &MutexGuard<'_, BTreeMap<usize, FigureData>>,
    user_id: &Arc<str>,
    ids: BTreeSet<usize>,
) -> (BTreeSet<usize>, BTreeSet<usize>) {
    let mut accepted_set = BTreeSet::new();
    let mut rejected_set = BTreeSet::new();

    for id in ids {
        if figures_lock.contains_key(&id) {
            accepted_set.insert(id);
        } else {
            rejected_set.insert(id);
        }
    }

    let backup = accepted_set.clone();
    if let Some(item) = selected_figures_lock.get_mut(user_id) {
        item.append(&mut accepted_set);
    } else {
        selected_figures_lock.insert(user_id.clone(), accepted_set);
    }

    (backup, rejected_set)
}

fn unselect(
    selected_figures_lock: &mut MutexGuard<'_, BTreeMap<Arc<str>, BTreeSet<usize>>>,
    figures_lock: &MutexGuard<'_, BTreeMap<usize, FigureData>>,
    user_id: &Arc<str>,
    ids: BTreeSet<usize>,
) -> (BTreeSet<usize>, BTreeSet<usize>) {
    let mut accepted_set = BTreeSet::new();
    let mut rejected_set = BTreeSet::new();

    for id in ids {
        if figures_lock.contains_key(&id) {
            accepted_set.insert(id);
        } else {
            rejected_set.insert(id);
        }
    }

    if let Some(item) = selected_figures_lock.get_mut(user_id) {
        for id in accepted_set.iter() {
            item.remove(id);
        }
        if item.is_empty() {
            selected_figures_lock.remove(user_id);
        }
    } else {
        unreachable!()
    }

    (accepted_set, rejected_set)
}
