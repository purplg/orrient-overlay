use bevy::prelude::*;

use crate::structs::MapId;

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
    ChangingMaps,
    InGame,
}

fn map_exit_system(mut game_state: ResMut<NextState<GameState>>) {
    game_state.set(GameState::ChangingMaps);
}

fn map_enter_system(mut game_state: ResMut<NextState<GameState>>) {
    game_state.set(GameState::InGame);
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_state(AppState::ParsingMarkerPacks)
            .add_sub_state::<GameState>();
        app.add_systems(
            PostUpdate,
            map_exit_system.run_if(resource_exists_and_changed::<MapId>),
        );
        app.add_systems(OnEnter(GameState::ChangingMaps), map_enter_system);
    }
}
