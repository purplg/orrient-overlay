use bevy::prelude::Event;
use byteorder::{LittleEndian, ReadBytesExt};
use mumblelink_reader::mumble_link::{MumbleLinkData, Position, Vector3D};
use orrient_input::ActionEvent;
use serde::{Deserialize, Serialize};
use std::{io::Seek, net::Ipv4Addr};

#[derive(Event, Clone, Serialize, Deserialize, Debug)]
pub enum SocketMessage {
    MumbleLinkData(Box<MumbleLinkDataDef>),
    Action(ActionEvent),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GW2Context {
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
            server_address: {
                let addr = Ipv4Addr::new(
                    cursor.read_u8()?,
                    cursor.read_u8()?,
                    cursor.read_u8()?,
                    cursor.read_u8()?,
                );
                // 28 bits are reserved to include ipv6 support.
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

    pub fn map_open(&self) -> bool {
        self.ui_state & 0b00000001 != 0
    }

    pub fn compass_top_right(&self) -> bool {
        self.ui_state & 0b00000010 != 0
    }

    pub fn compass_rotation_enabled(&self) -> bool {
        self.ui_state & 0b00000100 != 0
    }

    pub fn game_focused(&self) -> bool {
        self.ui_state & 0b00001000 != 0
    }

    pub fn in_competitive_gamemode(&self) -> bool {
        self.ui_state & 0b00010000 != 0
    }

    pub fn textbox_focused(&self) -> bool {
        self.ui_state & 0b00100000 != 0
    }

    pub fn in_combat(&self) -> bool {
        self.ui_state & 0b01000000 != 0
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
struct Identity {
    pub name: String,
    pub profession: u8,
    pub spec: u8,
    pub race: u8,
    pub map_id: u32,
    // obsolete
    #[serde(skip)]
    pub _world_id: usize,
    pub team_color_id: usize,
    pub commander: bool,
    pub fov: f32,
    pub uisz: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Profession {
    Unknown = 0,
    Guardian = 1,
    Warrior = 2,
    Engineer = 3,
    Ranger = 4,
    Thief = 5,
    Elementalist = 6,
    Mesmer = 7,
    Necromancer = 8,
    Revenant = 9,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IdentityDef {
    pub name: String,
    pub profession: Profession,
    pub spec: u8,
    pub race: u8,
    pub map_id: u32,
    pub team_color_id: usize,
    pub commander: bool,
    pub fov: f32,
    pub uisz: u8,
}

impl Profession {
    fn from_u8(value: u8) -> Self {
        match value {
            1 => Profession::Guardian,
            2 => Profession::Warrior,
            3 => Profession::Engineer,
            4 => Profession::Ranger,
            5 => Profession::Thief,
            6 => Profession::Elementalist,
            7 => Profession::Mesmer,
            8 => Profession::Necromancer,
            9 => Profession::Revenant,
            _ => Profession::Unknown,
        }
    }
}

impl IdentityDef {
    fn from_string(value: String) -> Result<Self, serde_json::Error> {
        let identity: Identity = serde_json::from_str(value.as_str())?;
        Ok(Self {
            name: identity.name,
            profession: Profession::from_u8(identity.profession),
            spec: identity.spec,
            race: identity.race,
            map_id: identity.map_id,
            team_color_id: identity.team_color_id,
            commander: identity.commander,
            fov: identity.fov,
            uisz: identity.uisz,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MumbleLinkDataDef {
    pub ui_version: i64,
    pub ui_tick: i64,
    pub avatar: PositionDef,
    pub name: String,
    pub camera: PositionDef,
    pub identity: IdentityDef,
    pub context_len: i64,
    pub context: GW2Context,
    pub description: String,
}

impl MumbleLinkDataDef {
    pub fn from_data(value: MumbleLinkData) -> Result<MumbleLinkDataDef, serde_json::Error> {
        let context = GW2Context::from_bytes(value.context).unwrap();
        Ok(Self {
            ui_version: value.ui_version,
            ui_tick: value.ui_tick,
            avatar: value.avatar.into(),
            name: value.name,
            camera: value.camera.into(),
            identity: IdentityDef::from_string(value.identity)?,
            context_len: value.context_len,
            context,
            description: value.description,
        })
    }
}
