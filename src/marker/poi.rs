use bevy::prelude::*;
use bevy_mod_billboard::{
    plugin::BillboardPlugin, BillboardMeshHandle, BillboardTextBundle, BillboardTextureBundle,
    BillboardTextureHandle,
};

use crate::{
    link::MapId, parser::prelude::*, player::Player, ui::compass::marker::ShowOnCompass, WorldEvent,
};

use super::{LoadedMarkers, MarkerEvent};

pub(super) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BillboardPlugin);
        app.init_resource::<LoadedMarkers>();

        app.add_systems(Startup, setup);

        app.add_systems(
            PreUpdate,
            load_marker.run_if(resource_exists::<MarkerPacks>.and_then(on_event::<MarkerEvent>())),
        );
        app.add_systems(
            Update,
            disappear_nearby_system.run_if(on_event::<WorldEvent>()),
        );
        app.add_systems(
            Update,
            (show_pois_system, hide_pois_system).run_if(on_event::<MarkerEvent>()),
        );
        app.add_systems(
            Update,
            track_loaded_system.run_if(on_event::<MarkerEvent>()),
        );
    }
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(PoiQuad(meshes.add(Rectangle::from_size(Vec2::splat(2.0)))));
}

fn load_marker(mut events: EventReader<MarkerEvent>, mut loaded_markers: ResMut<LoadedMarkers>) {
    for event in events.read() {
        let MarkerEvent::ShowMarker(full_id) = event else {
            continue;
        };

        loaded_markers.insert(full_id.clone());
    }
}

#[derive(Component)]
struct Poi(FullMarkerId);

#[derive(Component)]
struct DisappearNearby;

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

fn show_pois_system(
    mut commands: Commands,
    mut marker_events: EventReader<MarkerEvent>,
    packs: Res<MarkerPacks>,
    assets: Res<PoiQuad>,
    map_id: Option<Res<MapId>>,
) {
    for event in marker_events.read() {
        let MarkerEvent::ShowMarker(full_id) = event else {
            return;
        };

        info!("Loading POIs from {full_id}");

        let Some(pack) = &packs.get(&full_id.pack_id) else {
            warn!("Pack ID not found: {}", &full_id.pack_id);
            return;
        };

        let Some(marker) = &pack.get(&full_id.marker_id) else {
            warn!("Marker {full_id} not found in {}", full_id.pack_id);
            return;
        };

        let Some(pois) = pack.get_pois(&full_id.marker_id) else {
            info!("No POIs found for {}", full_id);
            return;
        };

        let pois = pois.iter().filter(|poi| {
            if let Some(map_id) = &map_id {
                poi.map_id == Some(***map_id)
            } else {
                true
            }
        });

        let mut count = 0;
        for poi in pois {
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

            let mut builder = commands.spawn(Poi(full_id.clone()));
            if let Some(icon) = icon {
                builder.insert(ShowOnCompass(icon.clone()));
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
                        "Unknown icon",
                        TextStyle {
                            font_size: 64.,
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

            count += 1;
        }

        info!("Loaded {} POIs.", count);
    }
}

fn hide_pois_system(
    mut commands: Commands,
    mut marker_events: EventReader<MarkerEvent>,
    poi_query: Query<(Entity, &Poi)>,
) {
    for event in marker_events.read() {
        let mut count = 0;
        match event {
            MarkerEvent::HideMarker(marker_id) => {
                for (entity, poi) in &poi_query {
                    if poi.0 == *marker_id {
                        commands.entity(entity).despawn_recursive();
                        count += 1;
                    }
                }
            }
            MarkerEvent::HideAllMarkers => {
                for (entity, _) in &poi_query {
                    commands.entity(entity).despawn_recursive();
                    count += 1;
                }
            }
            _ => {}
        }
        if count > 0 {
            info!("Unloaded {} POIs.", count);
        }
    }
}

fn track_loaded_system(
    mut loaded_markers: ResMut<LoadedMarkers>,
    mut marker_events: EventReader<MarkerEvent>,
) {
    for event in marker_events.read() {
        match event {
            MarkerEvent::ShowMarker(full_id) => {
                loaded_markers.insert(full_id.clone());
            }
            MarkerEvent::HideMarker(full_id) => {
                loaded_markers.remove(full_id);
            }
            MarkerEvent::HideAllMarkers => {
                loaded_markers.clear();
            }
        }
    }
}
