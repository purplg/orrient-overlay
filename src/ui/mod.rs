mod compass;
mod debug_panel;
mod marker_list;

use bevy::prelude::*;

use compass::UiCompassWindowExt as _;
use debug_panel::UiDebugPanelExt as _;
use marker_list::window::UiMarkerWindowExt as _;
use sickle_ui::{
    ui_builder::{UiBuilderExt as _, UiRoot},
    widgets::prelude::*,
    SickleUiPlugin,
};

use crate::UiEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SickleUiPlugin);

        app.add_plugins(compass::Plugin);
        app.add_plugins(marker_list::Plugin);
        app.add_plugins(debug_panel::Plugin);

        app.add_systems(Startup, setup);
    }
}

#[derive(Component)]
struct OrrientMenuItem(pub UiEvent);

fn setup(mut commands: Commands) {
    let camera = commands
        .spawn(Camera3dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                ..default()
            },
            ..default()
        })
        .id();

    commands.ui_builder(UiRoot).container(
        (
            NodeBundle {
                background_color: Color::NONE.into(),
                ..default()
            },
            TargetCamera(camera),
        ),
        |container| {
            container.compass();
            container.marker_window();
            container.debug_panel();
        },
    );
}
