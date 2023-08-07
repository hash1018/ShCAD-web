use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::figure::FigureData;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    UserJoined(UserId),
    FigureAdded(usize, FigureData),
    ResponseInfo(ResponseType),
    UserLeft(UserId),
    NotifyUserMousePositionChanged(UserId, VecDeque<(f64, f64)>),
    FigureSelected(UserId, BTreeSet<usize>),
    FigureUnselectedAll(UserId),
    NotifySelectDragStarted(UserId, f64, f64),
    NotifySelectDragFinished(UserId),
    SelectedFiguresUpdated(UserId, Option<BTreeSet<usize>>, Option<BTreeSet<usize>>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ClientMessage {
    Leave,
    Join(RoomId, UserId),
    AddFigure(FigureData),
    RequestInfo(RequestType),
    NotifyMousePositionChanged(VecDeque<(f64, f64)>),
    SelectFigure(BTreeSet<usize>),
    UnselectFigureAll,
    NotifySelectDragStart(f64, f64),
    NotifySelectDragFinish,
    UpdateSelectedFigures(Option<BTreeSet<usize>>, Option<BTreeSet<usize>>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RequestType {
    CurrentFigures,
    CheckRoomExist(RoomId),
    CheckUserExist(RoomId, UserId),
    CurrentSharedUsers,
    CurrentSelectedFigures,
    CurrentSelectDragPositions,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ResponseType {
    CurrentFigures(BTreeMap<usize, FigureData>),
    CurrentSharedUsers(Vec<String>),
    ResponseRoomExist(bool),
    ResponseUserExist(Option<bool>),
    InvalidRequest(RequestType),
    CurrentSelectedFigures(BTreeMap<String, BTreeSet<usize>>),
    CurrentSelectDragPositions(BTreeMap<String, (f64, f64)>),
}

pub type RoomId = String;
pub type UserId = String;
