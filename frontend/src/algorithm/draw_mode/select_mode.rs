use std::{cell::RefCell, rc::Rc};

use crate::{
    algorithm::visitor::finder::Finder,
    pages::workspace::{data::FigureList, draw_area::data::DrawAreaData},
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
        figures: Rc<FigureList>,
    ) -> Option<ShouldAction> {
        let (x, y) = self.convert_figure_coordinates(&event, data);

        let found = Rc::new(RefCell::new(false));
        let finder = Finder::new(found.clone(), (x, y), data.coordinates().zoom_rate, 6.0);

        let figure_list = figures.list();
        let mut list_borrow_mut = figure_list.borrow_mut();

        for figure in list_borrow_mut.iter_mut() {
            figure.accept(&finder);
            if *found.borrow() {
                log::info!("found!");
                break;
            }
        }

        if !*found.borrow() {
            log::info!("not found");
        }

        None
    }

    fn mouse_mouse_event(
        &mut self,
        _event: web_sys::MouseEvent,
        _data: &mut DrawAreaData,
        _figures: Rc<FigureList>,
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
