use crate::{base::DrawOption, pages::workspace::draw_area::data::DrawAreaData};

use super::ShouldAction;

#[derive(Default)]
pub struct PanMode {
    prev_x: f64,
    prev_y: f64,
}

impl PanMode {
    pub fn new() -> Self {
        Self {
            prev_x: 0.0,
            prev_y: 0.0,
        }
    }
}

impl PanMode {
    pub fn mouse_press_event(
        &mut self,
        event: web_sys::MouseEvent,
        _data: &mut DrawAreaData,
    ) -> Option<Vec<ShouldAction>> {
        self.prev_x = event.offset_x() as f64;
        self.prev_y = event.offset_y() as f64;

        let actions = vec![ShouldAction::Rerender(DrawOption::Remain)];
        Some(actions)
    }

    pub fn mouse_mouse_event(
        &mut self,
        event: web_sys::MouseEvent,
        data: &mut DrawAreaData,
    ) -> Option<Vec<ShouldAction>> {
        let x = event.offset_x() as f64;
        let y = event.offset_y() as f64;

        let coordinates = data.coordinates();
        let h_pos = coordinates.scroll_h_pos;
        let v_pos = coordinates.scroll_v_pos;

        data.set_scroll_pos(h_pos + (self.prev_x - x), v_pos + (self.prev_y - y));

        self.prev_x = x;
        self.prev_y = y;

        let actions = vec![ShouldAction::Rerender(DrawOption::DrawAll)];

        Some(actions)
    }
}
