use std::rc::Rc;
use yew::{html, Callback, Component, Properties};

use crate::pages::workspace::{title_bar::shared_user::SharedUser, workspace::ChildRequestType};

use super::{data::SharedUsers, UpdateReason};

mod shared_user;

#[derive(Clone, PartialEq, Properties)]
pub struct TitleBarProps {
    pub handler: Callback<ChildRequestType>,
    pub show_chat: bool,
    pub update_reason: Option<UpdateReason>,
    pub shared_users: Rc<SharedUsers>,
}

pub enum TitleBarMessage {}

pub struct TitleBar {}

impl Component for TitleBar {
    type Message = TitleBarMessage;
    type Properties = TitleBarProps;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {}
    }

    fn changed(&mut self, ctx: &yew::Context<Self>, _old_props: &Self::Properties) -> bool {
        if let Some(update_reason) = &ctx.props().update_reason {
            matches!(
                update_reason,
                UpdateReason::UserJoined
                    | UpdateReason::UserLeft
                    | UpdateReason::GetCurrentSharedUsers
            )
        } else {
            false
        }
    }

    fn update(&mut self, _ctx: &yew::Context<Self>, _msg: Self::Message) -> bool {
        true
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let leave_button_clicked = ctx.props().handler.reform(|_| ChildRequestType::Leave);

        let show_chat = ctx.props().show_chat;
        let chat_button_clicked = ctx
            .props()
            .handler
            .reform(move |_| ChildRequestType::ShowChat(!show_chat));

        let mut list = Vec::new();
        for (index, user) in ctx.props().shared_users.list().borrow().iter().enumerate() {
            list.push(html!{ <SharedUser name={user.user_id().to_string()} color={user.color().unwrap()} {index}/>});
        }

        html!(
            <div style="height: 100%; overflow: hidden;">
                <button class="leave_button" onclick={leave_button_clicked}></button>
                <button class={chat_button_css(show_chat)} onclick={chat_button_clicked}></button>
                {list}
            </div>
        )
    }
}

fn chat_button_css(show_chat: bool) -> String {
    if show_chat {
        String::from("chat_button_selected")
    } else {
        String::from("chat_button")
    }
}
