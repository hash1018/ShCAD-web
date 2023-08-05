pub mod chat;
pub mod data;
pub mod draw_area;
pub mod title_bar;
pub mod tool_box;
#[allow(clippy::module_inception)]
pub mod workspace;

#[derive(Clone, PartialEq, Debug)]
pub enum UpdateReason {
    Init,
    FigureAdded,
    GetCurrentFigures,
    GetCurrentSharedUsers,
    GetCurrentSelectedFigures,
    UserJoined,
    UserLeft,
    ShowChat,
    ChangeMode,
    MousePositionChanged,
    FigureSelected,
    FigureUnselectedAll,
    SelectDragFinished,
    GetCurrentSelectDragPositions,
}
