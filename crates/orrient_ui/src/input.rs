use crate::UiEvent;
use orrient_core::prelude::AppState;
use orrient_input::Action;
use orrient_input::ActionEvent;

use bevy::input::ButtonState;
use bevy::prelude::*;

fn update(mut events: EventReader<ActionEvent>, mut ew_ui: EventWriter<UiEvent>) {
    for event in events.read() {
        match event {
            ActionEvent {
                action,
                state: ButtonState::Pressed,
            } => match action {
                Action::Modifier => {}
                Action::Menu => {
                    ew_ui.send(UiEvent::ToggleUI);
                }
                Action::Close => {
                    ew_ui.send(UiEvent::CloseUi);
                }
                Action::Overlay => {}
            },
            _ => {}
        }
    }
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(orrient_input::Plugin);
        app.add_systems(Update, update.run_if(in_state(AppState::Running)));
    }
}
