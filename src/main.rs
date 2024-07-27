mod camera;
mod console;
mod input;
mod link;
mod marker;
mod parser;
mod player;
mod ui;

use bevy::window::{CompositeAlphaMode, WindowResolution};
use bevy::{prelude::*, window::WindowLevel};
use link::MapId;
use parser::prelude::*;

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
    ShowMarker(FullMarkerId),
    HideMarker(FullMarkerId),
    HideAllMarkers,
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

    app.add_event::<WorldEvent>();
    app.add_event::<UiEvent>();

    app.insert_resource(ClearColor(Color::NONE));

    app.add_plugins(camera::Plugin);
    app.add_plugins(input::Plugin);
    app.add_plugins(link::Plugin);
    app.add_plugins(player::Plugin);
    app.add_plugins(ui::Plugin);
    app.add_plugins(marker::Plugin);
    app.add_plugins(console::Plugin);
    app.add_plugins(parser::Plugin);

    app.run();
}
