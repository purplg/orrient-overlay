use crate::prelude::*;

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

#[derive(Event, Clone, Debug)]
pub enum UiEvent {
    OpenUi,
    CloseUi,
    CompassSize(UVec2),
    PlayerPosition(Vec2),
    MapPosition(Vec2),
    MapScale(f32),
    MapOpen(bool),
}

#[derive(Event, Clone, Debug)]
pub enum MarkerEvent {
    Show(FullMarkerId),
    Hide(FullMarkerId),
    HideAll,
}

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WorldEvent>();
        app.add_event::<UiEvent>();
        app.add_event::<MarkerEvent>();
    }
}
