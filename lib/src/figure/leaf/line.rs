use serde::{Deserialize, Serialize};

use crate::{
    common::Color,
    figure::{Figure, Visitor},
};

use super::Leaf;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LineData {
    pub start_x: f64,
    pub start_y: f64,
    pub end_x: f64,
    pub end_y: f64,
    pub color: Color,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    data: LineData,
}

impl Leaf for Line {}

impl Figure for Line {
    fn accept(&mut self, visitor: &dyn Visitor) {
        visitor.visit_line(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn data(&self) -> crate::figure::FigureData {
        crate::figure::FigureData::Line(self.data.clone())
    }

    fn box_clone(&self) -> Box<dyn Figure> {
        Box::new(self.clone())
    }
}

impl Line {
    pub fn new(start_x: f64, start_y: f64, end_x: f64, end_y: f64, color: Color) -> Self {
        let data = LineData {
            start_x,
            start_y,
            end_x,
            end_y,
            color,
        };

        Self { data }
    }

    pub fn start_x(&self) -> f64 {
        self.data.start_x
    }

    pub fn start_y(&self) -> f64 {
        self.data.start_y
    }

    pub fn end_x(&self) -> f64 {
        self.data.end_x
    }

    pub fn end_y(&self) -> f64 {
        self.data.end_y
    }

    pub fn color(&self) -> Color {
        self.data.color
    }

    pub fn set_start_x(&mut self, start_x: f64) {
        self.data.start_x = start_x;
    }

    pub fn set_start_y(&mut self, start_y: f64) {
        self.data.start_y = start_y;
    }

    pub fn set_end_x(&mut self, end_x: f64) {
        self.data.end_x = end_x;
    }

    pub fn set_end_y(&mut self, end_y: f64) {
        self.data.end_y = end_y;
    }
}
