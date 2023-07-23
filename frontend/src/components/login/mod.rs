use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::{html, Component, Properties};
use yew::{Callback, NodeRef};

use crate::components::login::api::{api_check_room_exist, api_check_user_exist};

mod api;
pub enum LoginMessage {
    JoinButtonClicked,
    CreateRoomButtonClicked,
}

pub enum LoginNotifyMessage {
    EnterRoom(String, Option<String>),
}

#[derive(Clone, PartialEq, Properties)]
pub struct LoginProps {
    pub handler: Callback<LoginNotifyMessage>,
    pub room_id: Option<String>,
}

pub struct Login {
    user_name_ref: NodeRef,
    room_id_ref: NodeRef,
}

impl Component for Login {
    type Message = LoginMessage;
    type Properties = LoginProps;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {
            user_name_ref: NodeRef::default(),
            room_id_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        if ctx.props().room_id.is_some() {
            update_by_workspace(self, ctx, msg)
        } else {
            update_by_main(self, ctx, msg)
        }
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let join_button_clicked = ctx.link().callback(|_| LoginMessage::JoinButtonClicked);
        let create_button_clicked = ctx
            .link()
            .callback(|_| LoginMessage::CreateRoomButtonClicked);

        let hide_create_button = ctx.props().room_id.is_some();

        html! {
           <div style="text-align:center;">
                <h1 style="font-size: 44px;">{"Sketchdove"}</h1>
                <h3>{"Real-time whiteboard"}</h3>
                <div>
                    <label>{"Enter your name"}</label>
                    <br/>
                    <input id="username" ref={&self.user_name_ref} type="text" placeholder="username" />
               </div>
               <div/>
               if !hide_create_button {
                    <div>
                        <label>{"Enter room id"}</label>
                        <br/>
                        <input id="room_id" ref={&self.room_id_ref} type="text" placeholder="room id" />
                    </div>
               }
               <div>
                    <button onclick={join_button_clicked}> {"Join"} </button>
                    <br/>
                    if !hide_create_button {
                        <label>{"or"}</label>
                        <br/>
                        <label>{"Create room"}</label>
                        <br/>
                        <button onclick={create_button_clicked}> {"Create"} </button>
                    }
               </div>
           </div>
        }
    }
}

fn update_by_workspace(
    login: &mut Login,
    ctx: &yew::Context<Login>,
    msg: <Login as Component>::Message,
) -> bool {
    let user_name = login
        .user_name_ref
        .cast::<HtmlInputElement>()
        .unwrap()
        .value();

    let room_id = ctx.props().room_id.as_ref().unwrap().clone();

    if user_name.is_empty() {
        web_sys::window()
            .unwrap()
            .alert_with_message("Input!")
            .unwrap();
        return false;
    }

    if let LoginMessage::JoinButtonClicked = msg {
        let handler = ctx.props().handler.clone();
        spawn_local(async move {
            if let Ok(result) = api_check_room_exist(&room_id).await {
                if result {
                    if let Ok(result) = api_check_user_exist(&user_name, &room_id).await {
                        if !result {
                            handler.emit(LoginNotifyMessage::EnterRoom(user_name, Some(room_id)));
                        } else {
                            let str = format!("user_id {user_name} already exist");
                            web_sys::window().unwrap().alert_with_message(&str).unwrap();
                        }
                    }
                } else {
                    handler.emit(LoginNotifyMessage::EnterRoom(user_name, Some(room_id)));
                }
            } else {
                let str = "error occurs try again".to_string();
                web_sys::window().unwrap().alert_with_message(&str).unwrap();
            }
        });
    }

    false
}

fn update_by_main(
    login: &mut Login,
    ctx: &yew::Context<Login>,
    msg: <Login as Component>::Message,
) -> bool {
    let user_name = login
        .user_name_ref
        .cast::<HtmlInputElement>()
        .unwrap()
        .value();

    let room_id = login
        .room_id_ref
        .cast::<HtmlInputElement>()
        .map(|room_id| room_id.value());

    if let Some(room_id) = room_id.as_ref() {
        if user_name.is_empty() || room_id.is_empty() {
            web_sys::window()
                .unwrap()
                .alert_with_message("Input!")
                .unwrap();
            return false;
        }
    } else if user_name.is_empty() {
        web_sys::window()
            .unwrap()
            .alert_with_message("Input!")
            .unwrap();
        return false;
    }

    match msg {
        LoginMessage::JoinButtonClicked => {
            let handler = ctx.props().handler.clone();
            let room_id = room_id.unwrap();
            spawn_local(async move {
                if let Ok(result) = api_check_room_exist(&room_id).await {
                    if result {
                        if let Ok(result) = api_check_user_exist(&user_name, &room_id).await {
                            if !result {
                                handler
                                    .emit(LoginNotifyMessage::EnterRoom(user_name, Some(room_id)));
                            } else {
                                let str = format!("user_name {user_name} already exist");
                                web_sys::window().unwrap().alert_with_message(&str).unwrap();
                            }
                        } else {
                            let str = format!("cannot join, user_name {user_name}, error occurs");
                            web_sys::window().unwrap().alert_with_message(&str).unwrap();
                        }
                    } else {
                        let str = format!("room_id {room_id} doesn't exist");
                        web_sys::window().unwrap().alert_with_message(&str).unwrap();
                    }
                } else {
                    let str = "error occurs try again".to_string();
                    web_sys::window().unwrap().alert_with_message(&str).unwrap();
                }
            });
        }
        LoginMessage::CreateRoomButtonClicked => {
            let handler = ctx.props().handler.clone();
            let room_id = room_id.unwrap();
            spawn_local(async move {
                if let Ok(result) = api_check_room_exist(&room_id).await {
                    if !result {
                        handler.emit(LoginNotifyMessage::EnterRoom(user_name, Some(room_id)));
                    } else {
                        let str = format!("room_id {room_id} already exist");
                        web_sys::window().unwrap().alert_with_message(&str).unwrap();
                    }
                } else {
                    let str = "error occurs try again".to_string();
                    web_sys::window().unwrap().alert_with_message(&str).unwrap();
                }
            });
        }
    }

    false
}
