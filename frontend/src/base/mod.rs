use std::collections::{BTreeSet, VecDeque};

use lib::{common::Color, figure::Figure};
use strum_macros::EnumIter;

#[derive(PartialEq, Copy, Clone, Debug, EnumIter)]
pub enum DrawModeType {
    SelectMode,
    LineMode,
}

pub enum ShouldAction {
    Rerender(DrawOption),
    BackToSelect,
    AddFigure(Box<dyn Figure>),
    NotifyMousePositionChanged(VecDeque<(f64, f64)>),
    SelectFigure(BTreeSet<usize>),
    UnselectFigureAll,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum DrawOption {
    Remain,
    DrawAll,
}

pub const SELECTED_FIGURE_COLOR: Color = Color {
    r: 135,
    g: 206,
    b: 235,
    a: 255,
};
pub const TOTAL_SELECTED_FIGURE_COLOR_RECT: Color = Color {
    r: 30,
    g: 144,
    b: 255,
    a: 255,
};

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub top_left: (f64, f64),
    pub width: f64,
    pub height: f64,
}

impl Rect {
    pub fn new(top_left: (f64, f64), width: f64, height: f64) -> Self {
        Self {
            top_left,
            width,
            height,
        }
    }
}
