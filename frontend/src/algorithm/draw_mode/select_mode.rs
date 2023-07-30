use std::{cell::RefCell, collections::BTreeSet, rc::Rc};

use crate::{
    algorithm::visitor::finder::Finder,
    pages::workspace::{data::FigureMaintainer, draw_area::data::DrawAreaData},
};

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
        event: web_sys::MouseEvent,
        data: &mut DrawAreaData,
        figure_maintainer: Rc<RefCell<FigureMaintainer>>,
    ) -> Option<ShouldAction> {
        let (x, y) = self.convert_figure_coordinates(&event, data);

        let finder = Finder::new((x, y), data.coordinates().zoom_rate, 6.0);

        let mut f_m_borrow_mut = figure_maintainer.borrow_mut();

        if let Some(id) = f_m_borrow_mut.search(&finder) {
            if !f_m_borrow_mut.check_selected(id) {
                let mut ids = BTreeSet::new();
                ids.insert(id);
                return Some(ShouldAction::SelectFigure(ids));
            }
        } else if f_m_borrow_mut.selected_list_len() != 0 {
            return Some(ShouldAction::UnselectFigureAll);
        }

        None
    }

    fn mouse_mouse_event(
        &mut self,
        _event: web_sys::MouseEvent,
        _data: &mut DrawAreaData,
        _figures: Rc<RefCell<FigureMaintainer>>,
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
