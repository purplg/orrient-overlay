mod camera;
mod link;
mod player;
mod ui;

use bevy::window::{CompositeAlphaMode, WindowResolution};
use bevy::{
    prelude::*,
    window::{Cursor, WindowLevel},
};
use link::MumbleLinkEvent;

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
    app.add_plugins(link::Plugin);
    app.add_plugins(player::Plugin);
    app.add_plugins(ui::Plugin);

    app.add_systems(Update, input);

    app.run();
}

fn input(input: Res<ButtonInput<KeyCode>>, mut mumble_link_event: EventWriter<MumbleLinkEvent>) {
    if input.just_pressed(KeyCode::Escape) {
        mumble_link_event.send(MumbleLinkEvent::Toggle);
    }
}
