mod marker_button;
mod separator;
pub(super) mod tooltip;
pub(super) mod window;

use bevy::prelude::*;
use sickle_ui::prelude::*;

use window::UiMarkerWindowExt as _;

use crate::UiCamera;

fn spawn_ui(mut commands: Commands, ui_camera: Res<UiCamera>) {
    commands.ui_builder(UiRoot).container(
        (NodeBundle::default(), TargetCamera(ui_camera.0)),
        |container| {
            container.marker_window();
        },
    );
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(window::Plugin);
        app.add_plugins(tooltip::Plugin);
        app.add_plugins(marker_button::Plugin);
        app.add_plugins(separator::Plugin);

        app.add_systems(Startup, spawn_ui);
    }
}
