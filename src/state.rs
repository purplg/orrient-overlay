use bevy::prelude::*;

#[derive(States, Hash, PartialEq, Eq, Debug, Clone)]
pub enum AppState {
    ParsingMarkerPacks,
    LoadingMarkerPacks,
    WaitingForMumbleLink,
    Running,
}

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_state(AppState::ParsingMarkerPacks);
    }
}
