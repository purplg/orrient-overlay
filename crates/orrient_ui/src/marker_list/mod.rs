mod downloads;
pub(super) mod window;
mod installed;

use bevy::prelude::*;

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(window::Plugin);
        app.add_plugins(downloads::Plugin);
        app.add_plugins(installed::Plugin);
    }
}
