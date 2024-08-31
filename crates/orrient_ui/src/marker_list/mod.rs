mod marker_button;
mod separator;
pub(super) mod tooltip;
pub(super) mod window;

use bevy::prelude::*;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(window::Plugin);
        app.add_plugins(tooltip::Plugin);
        app.add_plugins(marker_button::Plugin);
        app.add_plugins(separator::Plugin);
    }
}
