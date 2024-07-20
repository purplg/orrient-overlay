use bevy::{input::ButtonState, prelude::*};
use orrient_input::{Action, ActionEvent};

use crate::UiEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(orrient_input::Plugin);
        app.add_systems(Update, update);
    }
}

fn update(mut events: EventReader<ActionEvent>, mut mumble_link_event: EventWriter<UiEvent>) {
    for event in events.read() {
        match event {
            ActionEvent {
                action,
                state: ButtonState::Pressed,
            } => match action {
                Action::Modifier => {}
                Action::Menu => {
                    mumble_link_event.send(UiEvent::OpenUi);
                }
                Action::Close => {
                    mumble_link_event.send(UiEvent::CloseUi);
                }
                Action::Overlay => {}
            },
            _ => {}
        }
    }
}
