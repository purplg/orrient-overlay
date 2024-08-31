mod events;
mod player;
mod state;
mod structs;

pub mod prelude {
    pub use super::events::*;
    pub use super::player::*;
    pub use super::state::*;
    pub use super::structs::*;
}

use bevy::prelude::*;

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(events::Plugin);
        app.add_plugins(player::Plugin);
        app.add_plugins(state::Plugin);
    }
}
