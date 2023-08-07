use std::cell::RefCell;

use lib::figure::{leaf::line::Line, Visitor};

use crate::{
    algorithm::math::{
        check_point_lies_inside_rect, check_point_lies_on_line, check_two_line_segments_intersect,
    },
    base::Rect,
};

pub struct Finder {
    found: RefCell<bool>,
    point: (f64, f64),
    zoom_rate: f64,
    tolerance: f64,
}

impl Finder {
    pub fn new(point: (f64, f64), zoom_rate: f64, tolerance: f64) -> Self {
        Finder {
            found: RefCell::new(false),
            point,
            zoom_rate,
            tolerance,
        }
    }

    pub fn found(&self) -> bool {
        *self.found.borrow()
    }
}

impl Visitor for Finder {
    fn visit_line(&self, line: &mut Line) {
        let tolerance = self.tolerance / self.zoom_rate;
        let start = (line.start_x(), line.start_y());
        let end = (line.end_x(), line.end_y());

        *self.found.borrow_mut() = check_point_lies_on_line(self.point, start, end, tolerance);
    }
}

pub struct DragRectFinder {
    found: RefCell<bool>,
    rect: Rect,
}

impl DragRectFinder {
    pub fn new(rect: Rect) -> Self {
        Self {
            found: RefCell::new(false),
            rect,
        }
    }

    pub fn found(&self) -> bool {
        *self.found.borrow()
    }

    pub fn clear_found(&self) {
        *self.found.borrow_mut() = false;
    }
}

impl Visitor for DragRectFinder {
    fn visit_line(&self, line: &mut Line) {
        let start = (line.start_x(), line.start_y());
        let end = (line.end_x(), line.end_y());

        if check_point_lies_inside_rect(start, self.rect, 0.0)
            && check_point_lies_inside_rect(end, self.rect, 0.0)
        {
            *self.found.borrow_mut() = true;
            return;
        }

        let bottom_right = (
            self.rect.top_left.0 + self.rect.width,
            self.rect.top_left.1 - self.rect.height,
        );

        if check_two_line_segments_intersect(
            start,
            end,
            self.rect.top_left,
            (self.rect.top_left.0, bottom_right.1),
        )
        .is_some()
        {
            *self.found.borrow_mut() = true;
            return;
        }
        if check_two_line_segments_intersect(
            start,
            end,
            self.rect.top_left,
            (bottom_right.0, self.rect.top_left.1),
        )
        .is_some()
        {
            *self.found.borrow_mut() = true;
            return;
        }
        if check_two_line_segments_intersect(
            start,
            end,
            (self.rect.top_left.0, bottom_right.1),
            bottom_right,
        )
        .is_some()
        {
            *self.found.borrow_mut() = true;
            return;
        }
        if check_two_line_segments_intersect(
            start,
            end,
            (bottom_right.0, self.rect.top_left.1),
            bottom_right,
        )
        .is_some()
        {
            *self.found.borrow_mut() = true;
        }
    }
}
