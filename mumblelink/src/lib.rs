use std::{io::Seek, net::Ipv4Addr};

use byteorder::{LittleEndian, ReadBytesExt};
use mumblelink_reader::mumble_link::{MumbleLinkData, Position, Vector3D};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum MumbleLinkMessage {
    MumbleLinkData(MumbleLinkDataDef),
    Toggle,
    Save,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GW2Context {
    pub unknown: u32,
    pub server_address: Ipv4Addr,
    pub map_id: u32,
    pub map_type: u32,
    pub shard_id: u32,
    pub instance: u32,
    pub build_id: u32,
    pub ui_state: u32,
    pub compass_width: u16,
    pub compass_height: u16,
    pub compress_rotation: f32,
    pub player_x: f32,
    pub player_y: f32,
    pub map_center_x: f32,
    pub map_center_y: f32,
    pub map_scale: f32,
    pub process_id: u32,
    pub mount_index: u8,
}

impl GW2Context {
    fn from_bytes(value: [u8; 256]) -> Result<Self, std::io::Error> {
        let mut cursor = std::io::Cursor::new(value);
        Ok(Self {
            unknown: cursor.read_u32::<LittleEndian>()?,
            server_address: {
                let addr = Ipv4Addr::new(
                    cursor.read_u8()?,
                    cursor.read_u8()?,
                    cursor.read_u8()?,
                    cursor.read_u8()?,
                );
                cursor.seek_relative(24)?;
                addr
            },
            map_id: cursor.read_u32::<LittleEndian>()?,
            map_type: cursor.read_u32::<LittleEndian>()?,
            shard_id: cursor.read_u32::<LittleEndian>()?,
            instance: cursor.read_u32::<LittleEndian>()?,
            build_id: cursor.read_u32::<LittleEndian>()?,
            ui_state: cursor.read_u32::<LittleEndian>()?,
            compass_width: cursor.read_u16::<LittleEndian>()?,
            compass_height: cursor.read_u16::<LittleEndian>()?,
            compress_rotation: cursor.read_f32::<LittleEndian>()?,
            player_x: cursor.read_f32::<LittleEndian>()?,
            player_y: cursor.read_f32::<LittleEndian>()?,
            map_center_x: cursor.read_f32::<LittleEndian>()?,
            map_center_y: cursor.read_f32::<LittleEndian>()?,
            map_scale: cursor.read_f32::<LittleEndian>()?,
            process_id: cursor.read_u32::<LittleEndian>()?,
            mount_index: cursor.read_u8()?,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PositionDef {
    pub position: Vector3D,
    pub front: Vector3D,
    pub top: Vector3D,
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MumbleLinkDataDef {
    pub ui_version: i64,
    pub ui_tick: i64,
    pub avatar: PositionDef,
    pub name: String,
    pub camera: PositionDef,
    pub identity: String,
    pub context_len: i64,
    pub context: GW2Context,
    pub description: String,
}

impl From<MumbleLinkData> for MumbleLinkDataDef {
    fn from(value: MumbleLinkData) -> Self {
        let context = GW2Context::from_bytes(value.context).unwrap();
        Self {
            ui_version: value.ui_version,
            ui_tick: value.ui_tick,
            avatar: value.avatar.into(),
            name: value.name,
            camera: value.camera.into(),
            identity: value.identity,
            context_len: value.context_len,
            context,
            description: value.description,
        }
    }
}
