use lib::common::Color;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::{html, Component, Context, NodeRef, Properties};

use crate::algorithm::visitor::drawer::fill_circle;

#[derive(Clone, PartialEq, Properties)]
pub struct SharedUserProps {
    pub name: String,
    pub color: Color,
    pub index: usize,
}

pub enum SharedUserMessage {}

pub struct SharedUser {
    node_ref: NodeRef,
}

impl Component for SharedUser {
    type Message = SharedUserMessage;
    type Properties = SharedUserProps;

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {
            node_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, _ctx: &yew::Context<Self>, _msg: Self::Message) -> bool {
        true
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();
        let context: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        canvas.set_width(canvas.client_width() as u32);
        canvas.set_height(canvas.client_height() as u32);

        fill_circle((24.0, 24.0), 16.0, &ctx.props().color, &context);
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        let node_ref_clone = self.node_ref.clone();
        let style = format!(
            "position: absolute; float: right; right: {0}px; width: 42px; height: 48px;",
            ctx.props().index * 42 + 4
        );
        html! (
            <canvas style={style} ref={node_ref_clone}/>
        )
    }
}
