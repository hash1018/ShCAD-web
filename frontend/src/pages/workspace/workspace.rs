use std::{collections::VecDeque, rc::Rc};

use lib::{figure::Figure, message::ServerMessage};
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
    data::{FigureList, SharedUser, SharedUsers},
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
    figures: Rc<FigureList>,
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
            figures: Rc::new(FigureList::new()),
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
        let figures = self.figures.clone();
        let update_reason = self.update_reason.clone();
        let shared_users = self.shared_users.clone();

        html! {
            <body>
                <div class="top"> <TitleBar {handler} {show_chat} /> </div>
                <div class="content">
                    <DrawArea handler = {handler_clone} {current_mode} {figures} {update_reason} {shared_users} />
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
        ServerMessage::FigureAdded(data) => {
            workspace.figures.push(data.into());
            Some(UpdateReason::FigureAdded)
        }
        ServerMessage::ResponseInfo(response_type) => match response_type {
            lib::message::ResponseType::CurrentFigures(datas) => {
                if datas.is_empty() {
                    None
                } else {
                    let mut vec = Vec::new();
                    for data in datas {
                        vec.push(data.into());
                    }
                    workspace.figures.append(vec);
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
            _ => None,
        },
        ServerMessage::UserJoined(user_id) => {
            if user_id == user_name().unwrap() {
                if let Some(wss) = workspace.wss.as_ref() {
                    wss.send(lib::message::ClientMessage::RequestInfo(
                        lib::message::RequestType::CurrentFigures,
                    ));

                    wss.send(lib::message::ClientMessage::RequestInfo(
                        lib::message::RequestType::CurrentSharedUsers,
                    ));
                }
                None
            } else {
                let new_user = SharedUser::new(user_id, false);
                workspace.shared_users.push(new_user);
                Some(UpdateReason::UserJoined)
            }
        }
        ServerMessage::UserLeft(user_id) => {
            workspace.shared_users.remove(user_id);
            Some(UpdateReason::UserLeft)
        }
        ServerMessage::NotifyUserMousePositionChanged(user_id, queue) => {
            workspace.shared_users.update_mouse_position(user_id, queue);
            Some(UpdateReason::MousePositionChanged)
        }
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
    };

    update_reason
}
