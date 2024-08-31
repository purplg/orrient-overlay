mod camera;
mod console;
mod input;
mod link;
mod marker;
mod parser;
mod ui;

use bevy::window::WindowLevel;
use bevy::window::{CompositeAlphaMode, WindowResolution};

pub mod prelude {
    pub use orrient_core::state::*;
    pub use orrient_core::events::*;
    pub use orrient_core::player::*;
    pub use orrient_core::state::AppState;
    pub use orrient_core::state::GameState;
    pub use crate::link::MapId;
    pub use crate::parser::pack::Behavior;
    pub use crate::parser::pack::FullMarkerId;
    pub use crate::parser::pack::Marker;
    pub use crate::parser::pack::MarkerId;
    pub use crate::parser::pack::MarkerKind;
    pub use crate::parser::pack::MarkerPack;
    pub use crate::parser::MarkerPacks;
    pub use crate::parser::PackId;
    pub use crate::ui::compass::marker::ShowOnCompass;
    pub use bevy::prelude::*;
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

    app.add_plugins(camera::Plugin);
    app.add_plugins(console::Plugin);
    app.add_plugins(input::Plugin);
    app.add_plugins(link::Plugin);
    app.add_plugins(marker::Plugin);
    app.add_plugins(parser::Plugin);
    app.add_plugins(ui::Plugin);

    app.run();
}
