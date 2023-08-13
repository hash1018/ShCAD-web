use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, VecDeque},
    rc::Rc,
};

use lib::{
    figure::Figure,
    message::{AcceptedType, NotifyType, ServerMessage},
};
use yew::{html, Component, Context, Properties};
use yew_agent::{Bridge, Bridged};
use yew_router::scope_ext::RouterScopeExt;

use crate::{
    base::DrawModeType,
    client::{event_bus::EventBus, websocket_service::WebsocketService},
    components::login::{Login, LoginNotifyMessage},
    pages::{
        app::{set_user_name, user_name, Route},
        workspace::{chat::Chat, draw_area::DrawArea, title_bar::TitleBar, tool_box::ToolBox},
    },
};

use super::{
    data::{FigureMaintainer, SharedUser, SharedUsers},
    UpdateReason,
};

pub enum WorkSpaceMessage {
    HandleServerMessage(ServerMessage),
    HandleChildRequest(ChildRequestType),
    RequestInit,
    HandleLoginNotifyMessage(LoginNotifyMessage),
}

pub enum ChildRequestType {
    Leave,
    ShowChat(bool),
    ChangeMode(DrawModeType),
    AddFigure(Box<dyn Figure>),
    NotifyMousePositionChanged(VecDeque<(f64, f64)>),
    SelectFigure(BTreeSet<usize>),
    UnselectFigureAll,
    NotifySelectDragStart(f64, f64),
    NotifySelectDragFinish,
    UpdateSelectedFigures(Option<BTreeSet<usize>>, Option<BTreeSet<usize>>),
    DeleteFigures(BTreeSet<usize>),
}

#[derive(Clone, PartialEq, Properties)]
pub struct WorkspaceProps {
    pub id: String,
}

pub struct Workspace {
    wss: Option<WebsocketService>,
    _event_bus: Option<Box<dyn Bridge<EventBus>>>,
    show_chat: bool,
    current_mode: DrawModeType,
    figure_maintainer: Rc<RefCell<FigureMaintainer>>,
    shared_users: Rc<SharedUsers>,
    logined: bool,
    update_reason: Option<UpdateReason>,
}

impl Component for Workspace {
    type Message = WorkSpaceMessage;
    type Properties = WorkspaceProps;

    fn create(ctx: &yew::Context<Self>) -> Self {
        let user_name = user_name();
        if user_name.is_some() {
            let link = ctx.link();
            link.send_message(WorkSpaceMessage::RequestInit);
        }

        Self {
            wss: None,
            _event_bus: None,
            show_chat: false,
            current_mode: DrawModeType::SelectMode,
            figure_maintainer: Rc::new(RefCell::new(FigureMaintainer::new())),
            shared_users: Rc::new(SharedUsers::new()),
            logined: false,
            update_reason: None,
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        if let Some(wss) = self.wss.as_ref() {
            wss.disconnect();
        }
    }

    #[allow(clippy::single_match)]
    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        self.update_reason = handle_message(self, ctx, msg);
        self.update_reason.is_some()
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        if self.logined {
            self.show_draw_area(ctx)
        } else {
            let handler = ctx
                .link()
                .callback(WorkSpaceMessage::HandleLoginNotifyMessage);
            html! {
               <div>
                   <Login {handler} room_id = {Some(ctx.props().id.clone())} />
               </div>
            }
        }
    }
}

impl Workspace {
    fn show_draw_area(&self, ctx: &yew::Context<Workspace>) -> yew::Html {
        let handler = ctx.link().callback(WorkSpaceMessage::HandleChildRequest);
        let show_chat = self.show_chat;
        let current_mode = self.current_mode;
        let handler_clone = handler.clone();
        let handler_clone2 = handler.clone();
        let figure_maintainer = self.figure_maintainer.clone();
        let update_reason = self.update_reason.clone();
        let shared_users = self.shared_users.clone();

        html! {
            <body>
                <div class="top"> <TitleBar {handler} {show_chat} /> </div>
                <div class="content">
                    <DrawArea handler = {handler_clone} {current_mode} {figure_maintainer} {update_reason} {shared_users} />
                    <div class="left"> <ToolBox handler = {handler_clone2} {current_mode} /> </div>
                    if show_chat {
                        <div class="chat_position"> <Chat /> </div>
                    }
                </div>
            </body>
        }
    }
}

fn init(ctx: &Context<Workspace>) -> (Option<WebsocketService>, Option<Box<dyn Bridge<EventBus>>>) {
    let wss = WebsocketService::new();
    wss.connect().unwrap();
    let callback = {
        let link = ctx.link().clone();
        move |e| link.send_message(WorkSpaceMessage::HandleServerMessage(e))
    };

    (Some(wss), Some(EventBus::bridge(Rc::new(callback))))
}

fn handle_message(
    workspace: &mut Workspace,
    ctx: &yew::Context<Workspace>,
    msg: WorkSpaceMessage,
) -> Option<UpdateReason> {
    let update_reason = match msg {
        WorkSpaceMessage::RequestInit => {
            let user_name = user_name().unwrap();
            let room_id = ctx.props().id.clone();

            (workspace.wss, workspace._event_bus) = init(ctx);
            workspace.logined = true;

            if let Some(wss) = workspace.wss.as_ref() {
                wss.send(lib::message::ClientMessage::Join(room_id, user_name));
            }

            Some(UpdateReason::Init)
        }
        WorkSpaceMessage::HandleServerMessage(server_message) => {
            handle_server_message(workspace, ctx, server_message)
        }
        WorkSpaceMessage::HandleChildRequest(request) => {
            handle_child_request(workspace, ctx, request)
        }
        WorkSpaceMessage::HandleLoginNotifyMessage(msg) => match msg {
            LoginNotifyMessage::EnterRoom(name, _room_id) => {
                set_user_name(Some(name));
                let link = ctx.link();
                link.send_message(WorkSpaceMessage::RequestInit);
                None
            }
        },
    };

    update_reason
}

fn handle_server_message(
    workspace: &mut Workspace,
    _ctx: &yew::Context<Workspace>,
    msg: ServerMessage,
) -> Option<UpdateReason> {
    let update_reason = match msg {
        ServerMessage::Notify(notify_message) => match notify_message {
            NotifyType::FigureAdded(id, data) => {
                workspace
                    .figure_maintainer
                    .borrow_mut()
                    .insert_to_default(id, data.into());
                Some(UpdateReason::FigureAdded)
            }
            NotifyType::UserJoined(user_id) => {
                let new_user = SharedUser::new(user_id, false);
                workspace.shared_users.push(new_user);
                Some(UpdateReason::UserJoined)
            }
            NotifyType::UserLeft(user_id) => {
                workspace.shared_users.remove(user_id);
                Some(UpdateReason::UserLeft)
            }
            NotifyType::UserMousePositionChanged(user_id, queue) => {
                workspace.shared_users.update_mouse_position(user_id, queue);
                Some(UpdateReason::MousePositionChanged)
            }
            NotifyType::FigureUnselectedAll(user_id) => {
                workspace
                    .figure_maintainer
                    .borrow_mut()
                    .unselect_all_by_another_user(user_id);
                Some(UpdateReason::FigureUnselectedAll)
            }
            NotifyType::SelectDragStarted(user_id, x, y) => {
                workspace
                    .shared_users
                    .set_select_drag_position(user_id, Some((x, y)));

                None
            }
            NotifyType::SelectDragFinished(user_id) => {
                if user_id != user_name().unwrap() {
                    workspace
                        .shared_users
                        .set_select_drag_position(user_id, None);
                    Some(UpdateReason::SelectDragFinished)
                } else {
                    unreachable!()
                }
            }
            NotifyType::FigureDeleted(deleted_ids) => {
                let mut f_m_borrow_mut = workspace.figure_maintainer.borrow_mut();
                f_m_borrow_mut.delete_to_default(&deleted_ids);
                f_m_borrow_mut.unselect(&deleted_ids);
                f_m_borrow_mut.try_unselect_by_all_users(&deleted_ids);

                Some(UpdateReason::FigureDeleted)
            }
            NotifyType::FigureSelected(user_id, ids) => {
                workspace
                    .figure_maintainer
                    .borrow_mut()
                    .select_by_another_user(user_id, ids);

                Some(UpdateReason::FigureSelected)
            }
            NotifyType::SelectedFiguresUpdated(
                user_id,
                new_selected_figures,
                new_unselected_figures,
            ) => {
                if user_id != user_name().unwrap() {
                    if let Some(new_selected_figures) = new_selected_figures {
                        workspace
                            .figure_maintainer
                            .borrow_mut()
                            .select_by_another_user(user_id.clone(), new_selected_figures);
                    }
                    if let Some(new_unselected_figures) = new_unselected_figures {
                        workspace
                            .figure_maintainer
                            .borrow_mut()
                            .unselect_by_another_user(user_id, new_unselected_figures);
                    }
                } else {
                    unreachable!()
                }
                Some(UpdateReason::SelectedFiguresUpdated)
            }
        },
        ServerMessage::Response(response_type) => match response_type {
            lib::message::ResponseType::CurrentFigures(datas) => {
                if datas.is_empty() {
                    None
                } else {
                    let mut tree = BTreeMap::new();
                    for (id, data) in datas {
                        tree.insert(id, data.into());
                    }
                    workspace
                        .figure_maintainer
                        .borrow_mut()
                        .append_to_default(tree);
                    Some(UpdateReason::GetCurrentFigures)
                }
            }
            lib::message::ResponseType::CurrentSharedUsers(mut users) => {
                let my_name = user_name().unwrap();
                if let Some(position) = users.iter().position(|name| *name == my_name) {
                    users.remove(position);
                    let me = SharedUser::new(my_name, true);
                    workspace.shared_users.push(me);

                    if users.is_empty() {
                        None
                    } else {
                        let mut vec = Vec::new();
                        for user in users {
                            vec.push(SharedUser::new(user, false));
                        }

                        workspace.shared_users.append(vec);

                        Some(UpdateReason::GetCurrentSharedUsers)
                    }
                } else {
                    None
                }
            }
            lib::message::ResponseType::CurrentSelectedFigures(tree) => {
                if tree.is_empty() {
                    None
                } else {
                    let me = user_name().unwrap();
                    for (id, map) in tree {
                        if id != me {
                            workspace
                                .figure_maintainer
                                .borrow_mut()
                                .select_by_another_user(id, map);
                        }
                    }
                    Some(UpdateReason::GetCurrentSelectedFigures)
                }
            }
            lib::message::ResponseType::CurrentSelectDragPositions(tree) => {
                if tree.is_empty() {
                    None
                } else {
                    let me = user_name().unwrap();
                    for (id, position) in tree {
                        if id != me {
                            workspace
                                .shared_users
                                .set_select_drag_position(id, Some(position));
                        }
                    }
                    Some(UpdateReason::GetCurrentSelectDragPositions)
                }
            }
            _ => None,
        },
        ServerMessage::Accepted(accepted_type) => match accepted_type {
            AcceptedType::UserJoined => {
                if let Some(wss) = workspace.wss.as_ref() {
                    wss.send(lib::message::ClientMessage::RequestInfo(
                        lib::message::RequestType::CurrentSharedUsers,
                    ));

                    wss.send(lib::message::ClientMessage::RequestInfo(
                        lib::message::RequestType::CurrentFigures,
                    ));

                    wss.send(lib::message::ClientMessage::RequestInfo(
                        lib::message::RequestType::CurrentSelectedFigures,
                    ));

                    wss.send(lib::message::ClientMessage::RequestInfo(
                        lib::message::RequestType::CurrentSelectDragPositions,
                    ));
                }
                None
            }
            AcceptedType::FigureUnselectedAll => {
                workspace.figure_maintainer.borrow_mut().unselect_all();
                Some(UpdateReason::FigureUnselectedAll)
            }
            AcceptedType::FigureSelected(ids) => {
                workspace.figure_maintainer.borrow_mut().select(ids);
                Some(UpdateReason::FigureSelected)
            }
            AcceptedType::SelectedFiguresUpdated(new_selected_figures, new_unselected_figures) => {
                if let Some(new_selected_figures) = new_selected_figures {
                    workspace
                        .figure_maintainer
                        .borrow_mut()
                        .select(new_selected_figures);
                }
                if let Some(new_unselected_figures) = new_unselected_figures {
                    workspace
                        .figure_maintainer
                        .borrow_mut()
                        .unselect(&new_unselected_figures);
                }
                Some(UpdateReason::SelectedFiguresUpdated)
            }
            AcceptedType::FigureDeleted(ids) => {
                let mut f_m_borrow_mut = workspace.figure_maintainer.borrow_mut();
                f_m_borrow_mut.delete_to_default(&ids);
                f_m_borrow_mut.unselect(&ids);
                f_m_borrow_mut.try_unselect_by_all_users(&ids);

                Some(UpdateReason::FigureDeleted)
            }
        },
        ServerMessage::PartialAccepted(_, _) => None,
        ServerMessage::Rejected(_) => None,
    };

    update_reason
}

fn handle_child_request(
    workspace: &mut Workspace,
    ctx: &yew::Context<Workspace>,
    request: ChildRequestType,
) -> Option<UpdateReason> {
    let update_reason = match request {
        ChildRequestType::Leave => {
            let navigator = ctx.link().navigator().unwrap();
            navigator.push(&Route::Main);
            None
        }
        ChildRequestType::ShowChat(show) => {
            workspace.show_chat = show;
            Some(UpdateReason::ShowChat)
        }
        ChildRequestType::ChangeMode(mode) => {
            if mode != workspace.current_mode {
                workspace.current_mode = mode;
                //When change mode unselect all selected figures.
                if let Some(wss) = workspace.wss.as_ref() {
                    wss.send(lib::message::ClientMessage::UnselectFigureAll);
                }
                Some(UpdateReason::ChangeMode)
            } else {
                None
            }
        }
        ChildRequestType::AddFigure(figure) => {
            let data = figure.data();
            if let Some(wss) = workspace.wss.as_ref() {
                wss.send(lib::message::ClientMessage::AddFigure(data));
            }
            None
        }
        ChildRequestType::NotifyMousePositionChanged(queue) => {
            if let Some(wss) = workspace.wss.as_ref() {
                wss.send(lib::message::ClientMessage::NotifyMousePositionChanged(
                    queue,
                ));
            }
            None
        }
        ChildRequestType::SelectFigure(ids) => {
            if let Some(wss) = workspace.wss.as_ref() {
                wss.send(lib::message::ClientMessage::SelectFigure(ids));
            }
            None
        }
        ChildRequestType::UnselectFigureAll => {
            if let Some(wss) = workspace.wss.as_ref() {
                wss.send(lib::message::ClientMessage::UnselectFigureAll);
            }
            None
        }
        ChildRequestType::NotifySelectDragStart(x, y) => {
            if let Some(wss) = workspace.wss.as_ref() {
                wss.send(lib::message::ClientMessage::NotifySelectDragStart(x, y));
            }
            None
        }
        ChildRequestType::NotifySelectDragFinish => {
            if let Some(wss) = workspace.wss.as_ref() {
                wss.send(lib::message::ClientMessage::NotifySelectDragFinish);
            }
            None
        }
        ChildRequestType::UpdateSelectedFigures(about_to_select_set, about_to_unselect_set) => {
            if let Some(wss) = workspace.wss.as_ref() {
                wss.send(lib::message::ClientMessage::UpdateSelectedFigures(
                    about_to_select_set,
                    about_to_unselect_set,
                ));
            }
            None
        }
        ChildRequestType::DeleteFigures(ids) => {
            if let Some(wss) = workspace.wss.as_ref() {
                wss.send(lib::message::ClientMessage::DeleteFigures(ids));
            }
            None
        }
    };

    update_reason
}
