mod camera;
mod input;
mod link;
mod marker;
mod player;
mod trail;
mod ui;

use bevy::window::{CompositeAlphaMode, PrimaryWindow, WindowResolution};
use bevy::{
    prelude::*,
    window::{Cursor, WindowLevel},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[derive(Event)]
pub enum OrrientEvent {
    CameraUpdate {
        position: Vec3,
        facing: Vec3,
        fov: f32,
    },
    PlayerPositon(Vec3),
    ToggleUI,
    SavePosition,
    LoadMarker(String),
    LoadMarkers(String),
}

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "GW2Orrient".to_string(),
            resolution: WindowResolution::new(2560., 1440.),
            transparent: true,
            decorations: true,
            window_level: WindowLevel::AlwaysOnTop,
            composite_alpha_mode: CompositeAlphaMode::PreMultiplied,
            cursor: Cursor {
                hit_test: true,
                ..default()
            },
            ..default()
        }),
        ..default()
    }));

    // app.add_plugins(WorldInspectorPlugin::new());

    app.add_event::<OrrientEvent>();
    app.insert_resource(ClearColor(Color::NONE));

    app.add_plugins(camera::Plugin);
    app.add_plugins(input::Plugin);
    app.add_plugins(link::Plugin);
    app.add_plugins(player::Plugin);
    app.add_plugins(ui::Plugin);
    app.add_plugins(marker::Plugin);
    app.add_plugins(trail::Plugin);
    app.add_systems(Update, toggle_hittest_system);

    // app.world.send_event(OrrientEvent::ToggleUI);

    app.run();
}

fn toggle_hittest_system(
    mut events: EventReader<OrrientEvent>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
    for event in events.read() {
        if let OrrientEvent::ToggleUI = event {
            let mut window = window.single_mut();
            window.cursor.hit_test = !window.cursor.hit_test;

            window.decorations = window.cursor.hit_test;
            println!(
                "UI {}",
                if window.cursor.hit_test {
                    "enabled"
                } else {
                    "disabled"
                }
            );
        }
    }
}
