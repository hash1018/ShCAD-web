use std::collections::{BTreeSet, VecDeque};

use lib::figure::Figure;
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
