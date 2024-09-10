mod input;
mod link;

use bevy::window::WindowLevel;
use bevy::window::{CompositeAlphaMode, WindowResolution};

pub mod prelude {
    pub use bevy::prelude::*;
    pub use orrient_core::prelude::*;
    pub use orrient_pathing::prelude::*;
}

use crate::prelude::*;

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

    app.add_plugins(input::Plugin);
    app.add_plugins(link::Plugin);

    app.run();
}
