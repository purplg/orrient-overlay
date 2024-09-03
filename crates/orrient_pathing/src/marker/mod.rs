pub mod poi;
pub mod trail;

use orrient_core::prelude::*;

use bevy::prelude::*;
use bevy::utils::HashSet;

use crate::events::MarkerEvent;
use crate::parser::pack::FullMarkerId;
use crate::parser::MarkerPacks;

#[derive(Resource, Clone, Deref, DerefMut, Debug, Default)]
pub struct LoadedMarkers(pub HashSet<FullMarkerId>);

#[derive(Component)]
struct Marker(FullMarkerId);

/// Show which markers are valid for the current map
fn active_markers_system(
    packs: Res<MarkerPacks>,
    map_id: Res<MapId>,
    mut events: EventWriter<MarkerEvent>,
    mut loaded: ResMut<LoadedMarkers>,
) {
    loaded.0.clear();
    loaded.0.extend(packs.get_map_markers(&map_id.0));
    events.send_batch(
        loaded
            .0
            .iter()
            .map(|full_id| MarkerEvent::Show(full_id.clone())),
    );
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MarkerEvent>();

        app.add_plugins(poi::Plugin);
        app.add_plugins(trail::Plugin);

        app.add_systems(
            Update,
            active_markers_system
                .run_if(in_state(GameState::InGame))
                .run_if(resource_exists_and_changed::<MapId>),
        );
    }
}
