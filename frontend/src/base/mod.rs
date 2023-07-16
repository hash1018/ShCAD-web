use std::collections::VecDeque;

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
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum DrawOption {
    Remain,
    DrawAll,
}
