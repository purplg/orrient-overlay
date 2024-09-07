mod map_bounds;
pub mod marker;
pub mod window;

use bevy::prelude::*;
use sickle_ui::prelude::*;

use window::UiCompassWindowExt as _;

use crate::UiCamera;
use crate::UiEvent;

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

fn spawn_ui(mut commands: Commands, ui_camera: Res<UiCamera>) {
    commands.ui_builder(UiRoot).container(
        (NodeBundle::default(), TargetCamera(ui_camera.0)),
        |container| {
            container.compass();
        },
    );
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapOrientation>();
        app.add_plugins(marker::Plugin);
        app.add_plugins(window::Plugin);
        app.add_plugins(map_bounds::Plugin);

        app.add_systems(Startup, spawn_ui);
        app.add_systems(Update, map_system);
    }
}
