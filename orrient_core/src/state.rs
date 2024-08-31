use bevy::prelude::*;

#[derive(States, Hash, PartialEq, Eq, Debug, Clone)]
pub enum AppState {
    ParsingMarkerPacks,
    WaitingForMumbleLink,
    Running,
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(AppState = AppState::Running)]
pub enum GameState {
    Loading,
    WorldMap,
    #[default]
    InGame,
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_state(AppState::ParsingMarkerPacks)
            .add_sub_state::<GameState>();
    }
}
