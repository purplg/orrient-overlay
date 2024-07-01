mod camera;
mod console;
mod input;
mod link;
mod marker;
mod player;
mod trail;
#[path = "ui_sickle/mod.rs"]
mod ui;

use bevy::window::{CompositeAlphaMode, WindowResolution};
use bevy::{prelude::*, window::WindowLevel};

#[derive(Event, Clone, Debug)]
pub enum OrrientEvent {
    CameraUpdate {
        position: Vec3,
        facing: Vec3,
        fov: f32,
    },
    PlayerPositon(Vec3),
    ToggleUI,
    SavePosition,
    ShowMarkerBrowser,
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
            window_level: WindowLevel::AlwaysOnTop,
            composite_alpha_mode: CompositeAlphaMode::PreMultiplied,
            ..default()
        }),
        ..default()
    }));

    // app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin);
    // use bevy_inspector_egui::quick::WorldInspectorPlugin;
    // app.add_plugins(WorldInspectorPlugin::new());

    app.add_event::<OrrientEvent>();
    app.insert_resource(ClearColor(Color::NONE));

    app.add_plugins(camera::Plugin);
    app.add_plugins(input::Plugin);
    app.add_plugins(link::Plugin);
    app.add_plugins(player::Plugin);
    app.add_plugins(ui::Plugin);
    app.add_plugins(console::Plugin);
    app.add_plugins(marker::Plugin);
    app.add_plugins(trail::Plugin);

    app.run();
}
