use bevy::prelude::*;

use crate::OrrientEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update);
    }
}

fn update(input: Res<ButtonInput<KeyCode>>, mut mumble_link_event: EventWriter<OrrientEvent>) {
    if input.just_pressed(KeyCode::Escape) {
        mumble_link_event.send(OrrientEvent::ToggleUI);
    }
}
