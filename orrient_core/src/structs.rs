use bevy::prelude::*;
use std::ops::Deref;

#[derive(Resource, Clone, Copy, Debug)]
pub struct MapId(pub u32);

impl Deref for MapId {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
