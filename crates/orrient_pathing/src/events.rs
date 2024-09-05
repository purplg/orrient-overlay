use bevy::prelude::*;

use crate::parser::pack::FullMarkerId;

#[derive(Event, Clone, Debug)]
pub enum MarkerEvent {
    Enabled(FullMarkerId),
    Disable(FullMarkerId),
    DisableAll,
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MarkerEvent>();
    }
}
