pub mod events;
pub mod player;
pub mod state;

use bevy::prelude::*;

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(events::Plugin);
        app.add_plugins(player::Plugin);
        app.add_plugins(state::Plugin);
    }
}
