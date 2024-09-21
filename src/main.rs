use bevy::prelude::*;
use bevy::window::CompositeAlphaMode;
use bevy::window::WindowLevel;
use bevy::window::WindowResolution;

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

    app.add_plugins(orrient_api::Plugin);
    app.add_plugins(orrient_core::Plugin);
    app.add_plugins(orrient_pathing::Plugin);
    app.add_plugins(orrient_ui::Plugin);
    app.add_plugins(orrient_link::Plugin);

    app.run();
}
