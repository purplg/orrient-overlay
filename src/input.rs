use bevy::prelude::*;

use crate::UiEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update);
    }
}

fn update(input: Res<ButtonInput<KeyCode>>, mut mumble_link_event: EventWriter<UiEvent>) {
    if input.just_pressed(KeyCode::Escape) {
        mumble_link_event.send(UiEvent::ToggleUI);
    }
}
