use crate::pages::workspace::draw_area::data::DrawAreaData;

use super::{DrawMode, ShouldAction};

#[derive(Default)]
pub struct SelectMode {}

impl SelectMode {
    pub fn new() -> Self {
        SelectMode {}
    }
}

impl DrawMode for SelectMode {
    fn mouse_left_press_event(
        &mut self,
        _event: web_sys::MouseEvent,
        _data: &mut DrawAreaData,
    ) -> Option<ShouldAction> {
        None
    }

    fn mouse_mouse_event(
        &mut self,
        _event: web_sys::MouseEvent,
        _data: &mut DrawAreaData,
    ) -> Option<ShouldAction> {
        None
    }

    fn mouse_release_event(
        &mut self,
        _event: web_sys::MouseEvent,
        _data: &mut DrawAreaData,
    ) -> Option<ShouldAction> {
        None
    }

    fn get_type(&self) -> super::DrawModeType {
        super::DrawModeType::SelectMode
    }
}
