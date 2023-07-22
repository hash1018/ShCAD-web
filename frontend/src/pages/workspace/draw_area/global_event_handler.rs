use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::KeyboardEvent;

use super::{DrawArea, DrawAreaMessage};

pub struct GlobalEventHandler {
    keydown_closure: Option<Closure<dyn FnMut(KeyboardEvent)>>,
    visibilitychange_closure: Option<Closure<dyn FnMut()>>,
}

impl GlobalEventHandler {
    pub fn new() -> Self {
        GlobalEventHandler {
            keydown_closure: None,
            visibilitychange_closure: None,
        }
    }

    pub fn init(&mut self, ctx: &yew::Context<DrawArea>) {
        let link = ctx.link().clone();
        let closure = move |event| {
            link.send_message(DrawAreaMessage::KeyDown(event));
        };
        self.keydown_closure = add_event_listener("keydown", closure);

        let link = ctx.link().clone();
        let closure = move || {
            link.send_message(DrawAreaMessage::VisibilityChange(
                !web_sys::window().unwrap().document().unwrap().hidden(),
            ));
        };
        self.visibilitychange_closure =
            add_event_listener_without_return("visibilitychange", closure);
    }

    pub fn deinit(&mut self) {
        remove_event_listener("keydown", self.keydown_closure.take());
        remove_event_listener_without_return(
            "visibilitychange",
            self.visibilitychange_closure.take(),
        );
    }
}

fn add_event_listener<T, F>(event_type: &str, mut callback: F) -> Option<Closure<dyn FnMut(T)>>
where
    F: FnMut(T) + 'static,
    T: wasm_bindgen::convert::FromWasmAbi + 'static,
{
    if let Some(window) = web_sys::window() {
        let closure = Closure::<dyn FnMut(T)>::new(move |event: T| {
            callback(event);
        });
        window
            .add_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref())
            .unwrap();
        Some(closure)
    } else {
        None
    }
}

fn add_event_listener_without_return<F>(
    event_type: &str,
    mut callback: F,
) -> Option<Closure<dyn FnMut()>>
where
    F: FnMut() + 'static,
{
    if let Some(window) = web_sys::window() {
        let closure = Closure::<dyn FnMut()>::new(move || {
            callback();
        });
        window
            .add_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref())
            .unwrap();
        Some(closure)
    } else {
        None
    }
}

fn remove_event_listener<T>(event_type: &str, closure: Option<Closure<dyn FnMut(T)>>) {
    if let Some(window) = web_sys::window() {
        if let Some(closure) = closure {
            window
                .remove_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref())
                .expect("event_type and clsoure are not matched");
        }
    }
}

fn remove_event_listener_without_return(event_type: &str, closure: Option<Closure<dyn FnMut()>>) {
    if let Some(window) = web_sys::window() {
        if let Some(closure) = closure {
            window
                .remove_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref())
                .expect("event_type and clsoure are not matched");
        }
    }
}
