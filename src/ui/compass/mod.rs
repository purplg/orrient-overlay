mod marker;
mod window;

use crate::UiEvent;

pub use window::UiCompassWindowExt;

use bevy::{color::palettes, prelude::*, window::PrimaryWindow};
use sickle_ui::prelude::*;

#[derive(Resource, Default)]
struct MapOrientation {
    center: Vec2,
    scale: f32,
}

fn map_system(mut ui_events: EventReader<UiEvent>, mut orientation: ResMut<MapOrientation>) {
    for event in ui_events.read() {
        match event {
            UiEvent::MapPosition(position) => {
                orientation.center = *position;
            }
            UiEvent::MapScale(scale) => {
                orientation.scale = *scale;
            }
            _ => {}
        }
    }
}

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapOrientation>();
        app.add_plugins(marker::Plugin);
        app.add_plugins(window::Plugin);
        app.add_systems(Update, map_system);
    }
}
