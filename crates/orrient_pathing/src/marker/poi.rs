use bevy::prelude::*;
use orrient_core::prelude::*;

use super::LoadedMarkers;
use crate::events::MarkerEvent;
use crate::parser::pack::Behavior;
use crate::parser::MarkerPacks;

use bevy_mod_billboard::plugin::BillboardPlugin;
use bevy_mod_billboard::BillboardMeshHandle;
use bevy_mod_billboard::BillboardTextBundle;
use bevy_mod_billboard::BillboardTextureBundle;
use bevy_mod_billboard::BillboardTextureHandle;

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(PoiQuad(meshes.add(Rectangle::from_size(Vec2::splat(2.0)))));
}

#[derive(Component)]
pub struct PoiMarker;

#[derive(Component)]
pub struct DisappearNearby;

fn disappear_nearby_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<DisappearNearby>>,
    player: Query<&Transform, With<Player>>,
) {
    if let Ok(player) = player.get_single() {
        for (entity, transform) in &query {
            if transform.translation.distance_squared(player.translation) < 10. {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

#[derive(Resource)]
struct PoiQuad(Handle<Mesh>);

fn spawn_poi_system(
    mut commands: Commands,
    mut events: EventReader<MarkerEvent>,
    assets: Res<PoiQuad>,
    packs: Res<MarkerPacks>,
    map_id: Res<MapId>,
) {
    let mut count = 0;
    for event in events.read() {
        let MarkerEvent::Show(full_id) = event else {
            continue;
        };

        let Some(pack) = &packs.get(&full_id.pack_id) else {
            warn!("Pack ID not found: {}", &full_id.pack_id);
            continue;
        };

        let Some(pois) = pack.get_pois(&full_id.marker_id) else {
            continue;
        };

        let Some(marker) = &pack.get(&full_id.marker_id) else {
            warn!("Marker {full_id} not found in {}", full_id.pack_id);
            continue;
        };

        info!("Loading POIs from {full_id}");

        for poi in pois.iter().filter(|poi| poi.map_id == Some(map_id.0)) {
            let Some(pos) = poi.position.map(|position| Vec3 {
                x: position.x,
                y: position.y,
                z: -position.z,
            }) else {
                // No position recorded for this POI.
                continue;
            };

            let icon = poi
                .icon_file
                .clone()
                .or(pack
                    .get(&full_id.marker_id)
                    .and_then(|marker| marker.icon_file.clone()))
                .map(|icon_path| icon_path.into_string())
                .and_then(|path| pack.get_image(&path));

            let mut builder = commands.spawn_empty();
            if let Some(icon) = icon {
                builder.insert(BillboardTextureBundle {
                    mesh: BillboardMeshHandle(assets.0.clone()),
                    texture: BillboardTextureHandle(icon),
                    transform: Transform::from_translation(pos),
                    ..default()
                });
            } else {
                warn!("No icon for {}", full_id);
                builder.insert(BillboardTextBundle {
                    text: Text::from_section(
                        poi.id.0.to_string(),
                        TextStyle {
                            font_size: 32.,
                            ..default()
                        },
                    ),
                    transform: Transform::from_translation(pos).with_scale(Vec3::splat(0.01)),
                    ..default()
                });
            }

            if let Some(Behavior::ReappearDaily) = marker.behavior {
                builder.insert(DisappearNearby);
            } else if let Some(Behavior::DisappearOnUse) = marker.behavior {
                builder.insert(DisappearNearby);
            }

            builder.insert(PoiMarker);
            builder.insert(super::Marker(full_id.clone()));

            count += 1;
        }
    }

    if count > 0 {
        info!("Loaded {} POIs.", count);
    }
}

fn despawn_pois_system(
    mut commands: Commands,
    poi_query: Query<(Entity, &super::Marker), With<PoiMarker>>,
    mut events: EventReader<MarkerEvent>,
) {
    for event in events.read() {
        match event {
            MarkerEvent::Hide(full_id) => {
                let mut count = 0;
                for (entity, marker) in &poi_query {
                    if &marker.0 == full_id {
                        commands.entity(entity).despawn_recursive();
                        count += 1;
                    }
                }
                if count > 0 {
                    info!("Unloaded {} POIs.", count);
                }
            }
            MarkerEvent::HideAll => {
                let mut count = 0;
                for (entity, _marker) in &poi_query {
                    commands.entity(entity).despawn_recursive();
                    count += 1;
                }
                if count > 0 {
                    info!("Unloaded {} POIs.", count);
                }
            }
            MarkerEvent::Show(_) => {}
        }
    }
}

pub(super) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BillboardPlugin);
        app.init_resource::<LoadedMarkers>();

        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            disappear_nearby_system
                .run_if(in_state(GameState::InGame))
                .run_if(on_event::<WorldEvent>()),
        );
        app.add_systems(
            Update,
            (spawn_poi_system, despawn_pois_system)
                .chain()
                .run_if(in_state(GameState::InGame))
                .run_if(on_event::<MarkerEvent>()),
        );
    }
}
