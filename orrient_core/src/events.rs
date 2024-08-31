use bevy::prelude::*;

#[derive(Event, Clone, Debug)]
pub enum WorldEvent {
    CameraUpdate {
        position: Vec3,
        facing: Vec3,
        fov: f32,
    },
    PlayerPositon(Vec3),
    SavePosition,
}

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WorldEvent>();
    }
}
