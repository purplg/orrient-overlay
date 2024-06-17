mod camera;
mod link;
mod player;

use bevy::window::{CompositeAlphaMode, PrimaryWindow, WindowResolution};
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

    app.add_systems(
        Update,
        toggle_hittest_system.run_if(on_event::<MumbleLinkEvent>()),
    );
    app.add_systems(Update, input);

    app.run();
}

fn toggle_hittest_system(
    mut events: EventReader<MumbleLinkEvent>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
    for event in events.read() {
        if let MumbleLinkEvent::Toggle = event {
            let mut window = window.single_mut();
            window.cursor.hit_test = !window.cursor.hit_test;
            window.decorations = window.cursor.hit_test;
            println!("hittest: {:?}", window.cursor.hit_test);
        }
    }
}

fn input(input: Res<ButtonInput<KeyCode>>, mut mumble_link_event: EventWriter<MumbleLinkEvent>) {
    if input.just_pressed(KeyCode::Escape) {
        mumble_link_event.send(MumbleLinkEvent::Toggle);
    }
}
