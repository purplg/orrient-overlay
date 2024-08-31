use crate::prelude::*;
use bevy::input::ButtonState;
use orrient_input::Action;
use orrient_input::ActionEvent;
use orrient_ui::UiEvent;

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

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(orrient_input::Plugin);
        app.add_systems(Update, update.run_if(in_state(AppState::Running)));
    }
}
