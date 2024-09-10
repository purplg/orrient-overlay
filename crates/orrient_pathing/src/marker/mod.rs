pub mod poi;
pub mod trail;

use crate::events::MarkerEvent;
use crate::parser::pack::FullMarkerId;
use crate::parser::MarkerPacks;
use orrient_core::prelude::*;

use bevy::prelude::*;
use bevy::utils::HashSet;

use directories::BaseDirs;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write as _;

#[derive(Resource, Clone, Deref, DerefMut, Debug, Default)]
pub struct MapMarkers(pub HashSet<FullMarkerId>);

#[derive(Resource, Clone, Deref, DerefMut, Debug, Default, Serialize, Deserialize)]
pub struct EnabledMarkers(pub HashSet<FullMarkerId>);

#[derive(Component)]
struct Marker(FullMarkerId);

fn map_exit_system(mut map_markers: ResMut<MapMarkers>) {
    map_markers.0.clear();
}

fn map_enter_system(
    packs: Res<MarkerPacks>,
    map_id: Res<MapId>,
    mut map_markers: ResMut<MapMarkers>,
) {
    map_markers.0.extend(packs.get_map_markers(&map_id.0));
}

fn track_markers_system(
    mut events: EventReader<MarkerEvent>,
    mut enabled_markers: ResMut<EnabledMarkers>,
) {
    for event in events.read() {
        match event {
            MarkerEvent::Enabled(full_id) => {
                enabled_markers.insert(full_id.clone());
            }
            MarkerEvent::Disable(full_id) => {
                enabled_markers.remove(full_id);
            }
            MarkerEvent::DisableAll => {
                enabled_markers.clear();
            }
        }
    }
}

fn load_system(mut commands: Commands) {
    let Some(base_dirs) = BaseDirs::new() else {
        error!("Could not find base directories when trying to load.");
        commands.init_resource::<EnabledMarkers>();
        return;
    };

    let Some(state_dir) = base_dirs.state_dir() else {
        error!("Could not find state directory when trying to load.");
        commands.init_resource::<EnabledMarkers>();
        return;
    };

    let dir = state_dir.join("orrient");
    let filepath = dir.join("enabled.ron");

    let data = match File::open(filepath) {
        Ok(data) => data,
        Err(err) => {
            error!("Could not read enabled file when trying to load: {err:?}");
            commands.init_resource::<EnabledMarkers>();
            return;
        }
    };

    let enabled_markers: EnabledMarkers = match ron::de::from_reader(data) {
        Ok(inner) => inner,
        Err(err) => {
            error!("Could not deserialize enabled file when trying to load: {err:?}");
            commands.init_resource::<EnabledMarkers>();
            return;
        }
    };

    commands.insert_resource(enabled_markers);
}

fn save_system(enabled_markers: Res<EnabledMarkers>) {
    let Some(base_dirs) = BaseDirs::new() else {
        error!("Could not find base directories when trying to save.");
        return;
    };

    let Some(state_dir) = base_dirs.state_dir() else {
        error!("Could not find state directory when trying to save.");
        return;
    };

    let dir = state_dir.join("orrient");
    let filepath = dir.join("enabled.ron");

    info!("Saving enabled markers to {dir:?}");
    if let Err(err) = std::fs::create_dir_all(dir) {
        error!("Could not create state directory when trying to save: {err:?}");
        return;
    };

    let data = match ron::ser::to_string_pretty(&*enabled_markers, PrettyConfig::default()) {
        Ok(data) => data,
        Err(err) => {
            error!("Could not serialize enabled markers: {err:?}");
            return;
        }
    };

    match File::create(filepath) {
        Ok(mut file) => {
            file.write_all(data.as_bytes()).unwrap();
        }
        Err(err) => {
            error!("Could not write to state file when trying to save: {err:?}");
        }
    }
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MarkerEvent>();
        app.init_resource::<MapMarkers>();

        app.add_plugins(poi::Plugin);
        app.add_plugins(trail::Plugin);

        app.add_systems(OnEnter(GameState::ChangingMaps), map_exit_system);
        app.add_systems(OnEnter(GameState::InGame), map_enter_system);

        app.add_systems(Startup, load_system);

        app.add_systems(
            Update,
            save_system
                .run_if(in_state(GameState::InGame))
                .run_if(resource_exists_and_changed::<EnabledMarkers>),
        );

        app.add_systems(
            PostUpdate,
            track_markers_system
                .run_if(in_state(GameState::InGame))
                .run_if(on_event::<MarkerEvent>()),
        );
    }
}
