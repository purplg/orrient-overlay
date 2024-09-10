/// Input systems shared between both the overlay and link applications.

use bevy::{input::ButtonState, prelude::*};

use serde::{Deserialize, Serialize};

fn input_map(value: &KeyCode) -> Option<Action> {
    Some(match value {
        KeyCode::Tab => Action::Menu,
        KeyCode::Escape => Action::Close,
        KeyCode::ControlLeft => Action::Modifier,
        _ => return None,
    })
}

fn input_system(input: Res<ButtonInput<KeyCode>>, mut events: EventWriter<ActionEvent>) {
    let pressed = input
        .get_just_pressed()
        .filter_map(input_map)
        .map(ActionEvent::pressed);

    let released = input
        .get_just_released()
        .filter_map(input_map)
        .map(ActionEvent::released);

    events.send_batch(pressed.chain(released));
}

#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct ActionEvent {
    pub action: Action,
    pub state: ButtonState,
}

impl ActionEvent {
    fn pressed(action: Action) -> Self {
        Self {
            action,
            state: ButtonState::Pressed,
        }
    }

    fn released(action: Action) -> Self {
        Self {
            action,
            state: ButtonState::Released,
        }
    }

    pub fn is_pressed(&self) -> bool {
        self.state == ButtonState::Pressed
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Action {
    Modifier,
    Menu,
    Close,
    Overlay,
}

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ActionEvent>();
        app.add_systems(Update, input_system);
    }
}
