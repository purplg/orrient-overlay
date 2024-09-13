use bevy::prelude::*;

#[derive(Resource, Clone, Deref, DerefMut, Copy, Debug)]
pub struct MapId(pub u32);
