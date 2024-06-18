mod camera;
mod input;
mod link;
mod marker;
mod player;
mod ui;

use bevy::window::{CompositeAlphaMode, WindowResolution};
use bevy::{
    prelude::*,
    window::{Cursor, WindowLevel},
};

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "GW2Orrient".to_string(),
            resolution: WindowResolution::new(2560., 1440.),
            transparent: true,
            // decorations: false,
            window_level: WindowLevel::AlwaysOnTop,
            composite_alpha_mode: CompositeAlphaMode::PreMultiplied,
            cursor: Cursor {
                hit_test: false,
                ..default()
            },
            ..default()
        }),
        ..default()
    }));

    app.insert_resource(ClearColor(Color::NONE));

    app.add_plugins(camera::Plugin);
    app.add_plugins(input::Plugin);
    app.add_plugins(link::Plugin);
    app.add_plugins(player::Plugin);
    app.add_plugins(ui::Plugin);
    app.add_plugins(marker::Plugin);

    app.run();
}
