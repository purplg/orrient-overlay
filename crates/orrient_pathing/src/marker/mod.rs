pub mod poi;
pub mod trail;

use orrient_core::prelude::*;

use bevy::prelude::*;
use bevy::utils::HashSet;

use crate::events::MarkerEvent;
use crate::parser::pack::FullMarkerId;
use crate::parser::MarkerPacks;

#[derive(Resource, Clone, Deref, DerefMut, Debug, Default)]
pub struct MapMarkers(pub HashSet<FullMarkerId>);

#[derive(Resource, Clone, Deref, DerefMut, Debug, Default)]
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
            Update,
            track_markers_system
                .run_if(in_state(GameState::InGame))
                .run_if(on_event::<MarkerEvent>()),
        );
    }
}
