pub mod poi;
pub mod trail;

use crate::prelude::*;
use bevy::utils::HashSet;
use poi::LoadPoiEvent;
use trail::LoadTrailEvent;

#[derive(Resource, Clone, Deref, DerefMut, Debug, Default)]
pub struct LoadedMarkers(pub HashSet<FullMarkerId>);

#[derive(Event, Clone, Debug)]
pub enum MarkerEvent {
    Show(FullMarkerId),
    Hide(FullMarkerId),
    HideAll,
}

fn update_markers_system(
    packs: Res<MarkerPacks>,
    map_id: Res<MapId>,
    mut poi_events: EventWriter<LoadPoiEvent>,
    mut trail_events: EventWriter<LoadTrailEvent>,
    mut loaded: ResMut<LoadedMarkers>,
) {
    loaded.clear();
    loaded.extend(packs.get_map_markers(&map_id.0));
    poi_events.send_batch(loaded.iter().map(|full_id| LoadPoiEvent(full_id.clone())));
    trail_events.send_batch(loaded.iter().map(|full_id| LoadTrailEvent(full_id.clone())));
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(poi::Plugin);
        app.add_plugins(trail::Plugin);
        app.add_event::<MarkerEvent>();

        app.add_systems(
            Update,
            update_markers_system
                .run_if(in_state(GameState::InGame))
                .run_if(resource_exists_and_changed::<MapId>),
        );
    }
}
