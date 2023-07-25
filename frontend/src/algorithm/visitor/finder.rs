use std::{cell::RefCell, rc::Rc};

use lib::figure::{leaf::line::Line, Visitor};

use crate::algorithm::math::check_point_lies_on_line;

pub struct Finder {
    found: Rc<RefCell<bool>>,
    point: (f64, f64),
    zoom_rate: f64,
    tolerance: f64,
}

impl Finder {
    pub fn new(
        found: Rc<RefCell<bool>>,
        point: (f64, f64),
        zoom_rate: f64,
        tolerance: f64,
    ) -> Self {
        *found.borrow_mut() = false;
        Finder {
            found,
            point,
            zoom_rate,
            tolerance,
        }
    }
}

impl Visitor for Finder {
    fn visit_line(&self, line: &mut Line) {
        let tolerance = self.tolerance / self.zoom_rate;
        let start = (line.start_x(), line.start_y());
        let end = (line.end_x(), line.end_y());
        if check_point_lies_on_line(self.point, start, end, tolerance) {
            *self.found.borrow_mut() = true;
        }
    }
}
