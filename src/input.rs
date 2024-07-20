use bevy::prelude::*;
use orrient_input::Action;

use crate::UiEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(orrient_input::Plugin);
        app.add_systems(Update, update);
    }
}

fn update(mut events: EventReader<Action>, mut mumble_link_event: EventWriter<UiEvent>) {
    for event in events.read() {
        match event {
            Action::Menu => {
                mumble_link_event.send(UiEvent::ToggleUI);
            }
            Action::Overlay => {},
        }
    }
}
