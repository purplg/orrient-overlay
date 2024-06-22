pub mod marker_list;
pub use marker_list::*;

use bevy::prelude::*;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(marker_list::Plugin);
    }
}
