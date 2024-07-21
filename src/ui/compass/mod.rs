mod marker;
mod window;

pub use window::UiCompassWindowExt;

use bevy::{color::palettes, prelude::*, window::PrimaryWindow};
use sickle_ui::prelude::*;

use crate::UiEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(window::Plugin);
    }
}
