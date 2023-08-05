use std::cell::RefCell;

use lib::figure::{leaf::line::Line, Visitor};

use crate::{algorithm::math::caculate_rectangle, base::Rect};

pub struct RectPosGetter {
    rect: RefCell<Option<Rect>>,
}

impl Default for RectPosGetter {
    fn default() -> Self {
        Self::new()
    }
}

impl RectPosGetter {
    pub fn new() -> Self {
        RectPosGetter {
            rect: RefCell::new(None),
        }
    }

    pub fn rect(&self) -> Option<Rect> {
        *self.rect.borrow()
    }
}

impl Visitor for RectPosGetter {
    fn visit_line(&self, line: &mut Line) {
        let start = (line.start_x(), line.start_y());
        let end = (line.end_x(), line.end_y());

        let rect = caculate_rectangle(start, end, false);
        let right_x = rect.top_left.0 + rect.width;
        let bottom_y = rect.top_left.1 - rect.height;

        let mut rect_borrow_mut = self.rect.borrow_mut();

        if let Some(rect_tmp) = rect_borrow_mut.as_mut() {
            let mut right_x_tmp = rect_tmp.top_left.0 + rect_tmp.width;
            let mut bottom_y_tmp = rect_tmp.top_left.1 - rect_tmp.height;

            if rect.top_left.1 > rect_tmp.top_left.1 {
                rect_tmp.top_left.1 = rect.top_left.1;
            }

            if rect.top_left.0 < rect_tmp.top_left.0 {
                rect_tmp.top_left.0 = rect.top_left.0;
            }

            if right_x > right_x_tmp {
                right_x_tmp = right_x;
            }

            if bottom_y < bottom_y_tmp {
                bottom_y_tmp = bottom_y;
            }

            rect_tmp.width = right_x_tmp - rect_tmp.top_left.0;
            rect_tmp.height = rect_tmp.top_left.1 - bottom_y_tmp;
        } else {
            *rect_borrow_mut = Some(Rect::new(
                (rect.top_left.0, rect.top_left.1),
                rect.width,
                rect.height,
            ));
        }
    }
}
