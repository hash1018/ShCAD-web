use lib::figure::{leaf::line::Line, Color, Figure};

use crate::{base::DrawOption, components::draw_area::data::DrawAreaData};

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
    fn mouse_press_event(
        &mut self,
        event: web_sys::MouseEvent,
        data: &mut DrawAreaData,
    ) -> Option<ShouldAction> {
        let (x, y) = self.convert_figure_coordinates(&event, data);

        if let (Some(_), Some(_)) = (self.start_x.take(), self.start_y.take()) {
            if let Some(preview) = data.take_preview() {
                let preview = set_end_point_to_preview(preview, x, y);
                return Some(ShouldAction::AddFigure(preview));
            }
        } else {
            self.start_x = Some(x);
            self.start_y = Some(y);
            let line = Line::new(x, y, x, y, Color::new(0.0, 0.0, 0.0, 1.0));
            data.set_preview(Some(Box::new(line)));
        }
        None
    }

    fn mouse_mouse_event(
        &mut self,
        event: web_sys::MouseEvent,
        data: &mut DrawAreaData,
    ) -> Option<ShouldAction> {
        if self.start_x.is_some() && self.start_y.is_some() {
            let preview = data.take_preview();
            if let Some(preview) = preview {
                let (x, y) = self.convert_figure_coordinates(&event, data);
                let preview = set_end_point_to_preview(preview, x, y);
                data.set_preview(Some(preview));
                return Some(ShouldAction::Rerender(DrawOption::DrawAll));
            }
        }
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
