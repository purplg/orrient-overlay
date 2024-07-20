use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_input::prelude::*;
use serde::{Deserialize, Serialize};

fn input_system(input: Res<ButtonInput<KeyCode>>, mut events: EventWriter<Action>) {
    if input.just_pressed(KeyCode::Escape) {
        events.send(Action::Menu);
    }
}

#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub enum Action {
    Menu,
    Overlay,
}

pub struct Plugin;

impl bevy_app::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Action>();
        app.add_systems(Update, input_system);
    }
}
