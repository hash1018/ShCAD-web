use std::cell::RefCell;

use lib::figure::{leaf::line::Line, Visitor};

use crate::algorithm::math::caculate_rectangle;

pub struct RectPosGetter {
    top_y: RefCell<Option<f64>>,
    left_x: RefCell<Option<f64>>,
    width: RefCell<Option<f64>>,
    height: RefCell<Option<f64>>,
}

impl Default for RectPosGetter {
    fn default() -> Self {
        Self::new()
    }
}

impl RectPosGetter {
    pub fn new() -> Self {
        RectPosGetter {
            top_y: RefCell::new(None),
            left_x: RefCell::new(None),
            width: RefCell::new(None),
            height: RefCell::new(None),
        }
    }

    pub fn top_y(&self) -> Option<f64> {
        *self.top_y.borrow()
    }

    pub fn left_x(&self) -> Option<f64> {
        *self.left_x.borrow()
    }

    pub fn width(&self) -> Option<f64> {
        *self.width.borrow()
    }

    pub fn height(&self) -> Option<f64> {
        *self.height.borrow()
    }
}

impl Visitor for RectPosGetter {
    fn visit_line(&self, line: &mut Line) {
        let start = (line.start_x(), line.start_y());
        let end = (line.end_x(), line.end_y());

        let (top_left, width, height) = caculate_rectangle(start, end, false);
        let right_x = top_left.0 + width;
        let bottom_y = top_left.1 - height;

        let mut top_y_borrow_mut = self.top_y.borrow_mut();
        let mut left_x_borrow_mut = self.left_x.borrow_mut();
        let mut width_borrow_mut = self.width.borrow_mut();
        let mut height_borrow_mut = self.height.borrow_mut();

        if let (Some(top_y), Some(left_x), Some(width), Some(height)) = (
            top_y_borrow_mut.as_mut(),
            left_x_borrow_mut.as_mut(),
            width_borrow_mut.as_mut(),
            height_borrow_mut.as_mut(),
        ) {
            let mut right_x_tmp = *left_x + *width;
            let mut bottom_y_tmp = *top_y - *height;

            if top_left.1 > *top_y {
                *top_y = top_left.1;
            }

            if top_left.0 < *left_x {
                *left_x = top_left.0;
            }

            if right_x > right_x_tmp {
                right_x_tmp = right_x;
            }

            if bottom_y < bottom_y_tmp {
                bottom_y_tmp = bottom_y;
            }

            *width = right_x_tmp - *left_x;
            *height = *top_y - bottom_y_tmp;
        } else {
            *top_y_borrow_mut = Some(top_left.1);
            *left_x_borrow_mut = Some(top_left.0);
            *width_borrow_mut = Some(width);
            *height_borrow_mut = Some(height);
        }
    }
}
