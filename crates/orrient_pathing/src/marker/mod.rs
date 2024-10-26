pub mod poi;
pub mod trail;

use crate::events::MarkerEvent;
use crate::parser::pack::FullMarkerId;
use crate::parser::MarkerPacks;
use anyhow::{anyhow, Result};
use orrient_core::prelude::*;

use bevy::prelude::*;
use bevy::utils::HashSet;

use directories::BaseDirs;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write as _;
use std::path::PathBuf;

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
            MarkerEvent::Enable(full_id) => {
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

fn find_enabled_file() -> Result<PathBuf> {
    let base_dirs = BaseDirs::new().ok_or(anyhow!("Could not find base directories"))?;

    let state_dir = base_dirs
        .state_dir()
        .ok_or(anyhow!("Could not find state directory"))?;

    let dir = state_dir.join("orrient");
    std::fs::create_dir_all(&dir)
        .map_err(|err| anyhow!("Could not create directory {dir:?}: {err:?}"))?;

    Ok(dir.join("enabled.ron"))
}

fn load_system(filepath: In<Result<PathBuf>>, mut commands: Commands) -> Result<()> {
    let filepath = filepath.0?;

    if !std::fs::exists(&filepath).unwrap_or_default() {
        commands.init_resource::<EnabledMarkers>();
        return Ok(());
    }

    let data =
        File::open(&filepath).map_err(|err| anyhow!("Could not read {filepath:?}: {err:?}"))?;

    let enabled_markers: EnabledMarkers = ron::de::from_reader(data)
        .map_err(|err| anyhow!("Could not deserialize {filepath:?}: {err:?}"))?;

    commands.insert_resource(enabled_markers);
    Ok(())
}

fn save_system(filepath: In<Result<PathBuf>>, enabled_markers: Res<EnabledMarkers>) -> Result<()> {
    let filepath = filepath.0?;
    info!("Saving enabled markers to {filepath:?}");

    let data = ron::ser::to_string_pretty(&*enabled_markers, PrettyConfig::default())
        .map_err(|err| anyhow!("Could not serialize enabled markers: {err:?}"))?;

    let mut file = File::create(&filepath)
        .map_err(|err| anyhow!("Could not write to state file when trying to save: {err:?}"))?;

    file.write_all(data.as_bytes())
        .map_err(|err| anyhow!("Could not write to {filepath:?}: {err:?}"))?;

    Ok(())
}

fn output_error(result: In<Result<()>>) {
    if let Err(err) = result.0 {
        error!("{err:?}");
    }
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MarkerEvent>();
        app.init_resource::<MapMarkers>();
        app.init_resource::<EnabledMarkers>();

        app.add_plugins(poi::Plugin);
        app.add_plugins(trail::Plugin);

        app.add_systems(OnEnter(GameState::ChangingMaps), map_exit_system);
        app.add_systems(OnEnter(GameState::InGame), map_enter_system);
        app.add_systems(
            Startup,
            find_enabled_file.pipe(load_system).pipe(output_error),
        );
        app.add_systems(
            Update,
            find_enabled_file
                .pipe(save_system)
                .pipe(output_error)
                .run_if(not(in_state(AppState::ParsingMarkerPacks)))
                .run_if(resource_exists_and_changed::<EnabledMarkers>),
        );
        app.add_systems(
            PostUpdate,
            track_markers_system.run_if(on_event::<MarkerEvent>()),
        );
    }
}
