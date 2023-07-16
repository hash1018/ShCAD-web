use std::collections::VecDeque;

use serde::{Deserialize, Serialize};

use crate::figure::FigureData;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    UserJoined(UserId),
    FigureAdded(FigureData),
    ResponseInfo(ResponseType),
    UserLeft(UserId),
    NotifyUserMousePositionChanged(UserId, VecDeque<(f64, f64)>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ClientMessage {
    Leave,
    Join(RoomId, UserId),
    AddFigure(FigureData),
    RequestInfo(RequestType),
    NotifyMousePositionChanged(VecDeque<(f64, f64)>),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum RequestType {
    CurrentFigures,
    CheckRoomExist(RoomId),
    CheckUserExist(RoomId, UserId),
    CurrentSharedUsers,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ResponseType {
    CurrentFigures(Vec<FigureData>),
    CurrentSharedUsers(Vec<String>),
    ResponseRoomExist(bool),
    ResponseUserExist(Option<bool>),
    InvalidRequest(RequestType),
}

pub type RoomId = String;
pub type UserId = String;
