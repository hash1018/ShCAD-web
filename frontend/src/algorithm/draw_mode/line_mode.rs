use std::{cell::RefCell, rc::Rc};

use lib::{
    common::Color,
    figure::{leaf::line::Line, Figure},
};

use crate::{
    base::DrawOption,
    pages::workspace::{data::FigureMaintainer, draw_area::data::DrawAreaData},
};

use super::{DrawMode, ShouldAction};

#[derive(Default)]
pub struct LineMode {
    start_x: Option<f64>,
    start_y: Option<f64>,
}

impl LineMode {
    pub fn new() -> Self {
        LineMode {
            start_x: None,
            start_y: None,
        }
    }
}

impl DrawMode for LineMode {
    fn mouse_left_press_event(
        &mut self,
        event: web_sys::MouseEvent,
        data: &mut DrawAreaData,
        figure_maintainer: Rc<RefCell<FigureMaintainer>>,
    ) -> Option<Vec<ShouldAction>> {
        let (x, y) = self.convert_figure_coordinates(&event, data);

        if let (Some(_), Some(_)) = (self.start_x.take(), self.start_y.take()) {
            if let Some(preview) = figure_maintainer.borrow_mut().take_preview() {
                let preview = set_end_point_to_preview(preview, x, y);
                return Some(vec![ShouldAction::AddFigure(preview)]);
            }
        } else {
            self.start_x = Some(x);
            self.start_y = Some(y);
            let line = Line::new(x, y, x, y, Color::new(0, 0, 0, 255));
            figure_maintainer
                .borrow_mut()
                .set_preview(Some(Box::new(line)));
        }
        None
    }

    fn mouse_mouse_event(
        &mut self,
        event: web_sys::MouseEvent,
        data: &mut DrawAreaData,
        figure_maintainer: Rc<RefCell<FigureMaintainer>>,
    ) -> Option<Vec<ShouldAction>> {
        if self.start_x.is_some() && self.start_y.is_some() {
            let preview = figure_maintainer.borrow_mut().take_preview();
            if let Some(preview) = preview {
                let (x, y) = self.convert_figure_coordinates(&event, data);
                let preview = set_end_point_to_preview(preview, x, y);
                figure_maintainer.borrow_mut().set_preview(Some(preview));
                return Some(vec![ShouldAction::Rerender(DrawOption::DrawAll)]);
            }
        }
        None
    }

    fn mouse_release_event(
        &mut self,
        _event: web_sys::MouseEvent,
        _data: &mut DrawAreaData,
    ) -> Option<Vec<ShouldAction>> {
        None
    }

    fn get_type(&self) -> super::DrawModeType {
        super::DrawModeType::LineMode
    }
}

fn set_end_point_to_preview(mut preview: Box<dyn Figure>, x: f64, y: f64) -> Box<dyn Figure> {
    let preview_tmp = preview.as_any_mut();
    if let Some(line) = preview_tmp.downcast_mut::<Line>() {
        line.set_end_x(x);
        line.set_end_y(y);
    }
    preview
}
