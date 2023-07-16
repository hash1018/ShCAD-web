use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use gloo_timers::callback::Interval;
use yew::html::Scope;

use super::{DrawArea, DrawAreaMessage};

#[derive(Default)]
pub struct MouseTracker {
    queue: Rc<RefCell<VecDeque<(f64, f64)>>>,
    interval: Option<Interval>,
}

impl MouseTracker {
    pub fn new() -> Self {
        MouseTracker {
            queue: Rc::new(RefCell::new(VecDeque::new())),
            interval: None,
        }
    }

    pub fn set_current_pos(&mut self, x: f64, y: f64) {
        self.queue.borrow_mut().push_back((x, y));
    }

    pub fn run(&mut self, link: Scope<DrawArea>) {
        let queue_clone = self.queue.clone();

        let interval = Interval::new(200, move || {
            let len = queue_clone.borrow().len();
            if len > 0 {
                if len > 20 {}
                let queue = queue_clone.borrow().clone();
                link.send_message(DrawAreaMessage::MousePositionChanged(queue));
                queue_clone.borrow_mut().clear();
            }
        });

        self.interval = Some(interval);
    }

    pub fn stop(&mut self) {
        self.interval.take();
    }
}
