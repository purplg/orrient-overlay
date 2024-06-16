use mumblelink_reader::mumble_link::{MumbleLinkData, Position, Vector3D};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PositionDef {
    pub position: Vector3D,
    pub front: Vector3D,
    pub top: Vector3D,
}

impl Into<Position> for PositionDef {
    fn into(self) -> Position {
        Position {
            position: self.position,
            front: self.front,
            top: self.top,
        }
    }
}

impl From<Position> for PositionDef {
    fn from(value: Position) -> Self {
        Self {
            position: value.position,
            front: value.front,
            top: value.top,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MumbleLinkDataDef {
    pub ui_version: i64,
    pub ui_tick: i64,
    pub avatar: PositionDef,
    pub name: String,
    pub camera: PositionDef,
    pub identity: String,
    pub context_len: i64,
    pub context: Vec<u8>,
    pub description: String,
}

impl Into<MumbleLinkData> for MumbleLinkDataDef {
    fn into(self) -> MumbleLinkData {
        MumbleLinkData {
            ui_version: self.ui_version,
            ui_tick: self.ui_tick,
            avatar: self.avatar.into(),
            name: self.name,
            camera: self.camera.into(),
            identity: self.identity,
            context_len: self.context_len,
            context: self.context.try_into().unwrap(),
            description: self.description,
        }
    }
}

impl From<MumbleLinkData> for MumbleLinkDataDef {
    fn from(value: MumbleLinkData) -> Self {
        Self {
            ui_version: value.ui_version,
            ui_tick: value.ui_tick,
            avatar: value.avatar.into(),
            name: value.name,
            camera: value.camera.into(),
            identity: value.identity,
            context_len: value.context_len,
            context: value.context.into(),
            description: value.description,
        }
    }
}
