use yew::{html, Component};

pub struct Register {}

impl Component for Register {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &yew::Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, _ctx: &yew::Context<Self>, _msg: Self::Message) -> bool {
        true
    }

    fn view(&self, _ctx: &yew::Context<Self>) -> yew::Html {
        html! {
            <body> { "Hello register" } </body>
        }
    }
}