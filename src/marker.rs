use bevy::prelude::*;
use bevy_mod_billboard::{
    plugin::BillboardPlugin, BillboardMeshHandle, BillboardTextBundle, BillboardTextureBundle,
    BillboardTextureHandle,
};

use crate::{link::MapId, player::Player, trail::DebugMarkerAssets, UiEvent, WorldEvent};

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BillboardPlugin);
        app.init_resource::<LoadedMarkers>();

        app.add_systems(Startup, setup);
        app.add_systems(
            PreUpdate,
            load_marker.run_if(resource_exists::<MarkerTree>.and_then(on_event::<UiEvent>())),
        );
        app.add_systems(
            Update,
            disappear_nearby_system.run_if(on_event::<WorldEvent>()),
        );
        app.add_systems(
            Update,
            (load_pois_system, unload_pois_system).run_if(on_event::<UiEvent>()),
        );
    }
}

fn load_marker(mut events: EventReader<UiEvent>, mut markers: ResMut<LoadedMarkers>) {
    for event in events.read() {
        let UiEvent::LoadMarker(marker_id) = event else {
            return;
        };

        markers.push(marker_id.clone());
    }
}

fn setup(mut commands: Commands) {
    let markers =
        match crate::parser::read(&dirs::config_dir().unwrap().join("orrient").join("markers")) {
            Ok(markers) => markers,
            Err(err) => {
                println!("Error when loading markers: {:?}", err);
                return;
            }
        };

    commands.insert_resource(MarkerTree(markers));
}

#[derive(Component)]
struct Poi(String);

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

fn load_pois_system(
    mut commands: Commands,
    mut ui_events: EventReader<UiEvent>,
    data: Res<MarkerTree>,
    assets: Res<DebugMarkerAssets>,
    map_id: Option<Res<MapId>>,
    asset_server: Res<AssetServer>,
) {
    for event in ui_events.read() {
        let UiEvent::LoadMarker(marker_id) = event else {
            return;
        };

        info!("Loading POIs for {}", marker_id);

        let Some(marker) = &data.get(marker_id) else {
            warn!("Marker ID not found: {}", marker_id);
            return;
        };

        let Some(pois) = data.get_pois(&marker.id) else {
            info!("No POIs found for {}", marker_id);
            return;
        };

        let pois = pois.iter().filter(|poi| {
            if let Some(map_id) = &map_id {
                poi.map_id == ***map_id
            } else {
                true
            }
        });

        let mut count = 0;
        for poi in pois {
            let pos = Vec3 {
                x: poi.position.x,
                y: poi.position.y,
                z: -poi.position.z,
            };

            let icon = poi
                .icon_file
                .clone()
                .or(data
                    .get(marker_id)
                    .and_then(|marker| marker.icon_file.clone()))
                .and_then(|icon_path| {
                    dirs::config_dir()
                        .unwrap()
                        .join("orrient")
                        .join("markers")
                        .join(icon_path)
                        .into_os_string()
                        .into_string()
                        .ok()
                });

            let mut builder = commands.spawn(Poi(marker_id.to_string()));
            if let Some(icon) = icon {
                builder.insert(BillboardTextureBundle {
                    mesh: BillboardMeshHandle(assets.image_quad.clone()),
                    texture: BillboardTextureHandle(asset_server.load(icon)),
                    transform: Transform::from_translation(pos),
                    ..default()
                });
            } else {
                warn!("No icon for {}", marker_id);
            }

            debug!("Spawned POI at {}", pos);

            if let Some(crate::parser::Behavior::ReappearDaily) = marker.behavior {
                builder.insert(DisappearNearby);
            } else if let Some(crate::parser::Behavior::DisappearOnUse) = marker.behavior {
                builder.insert(DisappearNearby);
            }
            count += 1;
        }

        info!("Loaded {} POIs.", count);
    }
}

fn unload_pois_system(
    mut commands: Commands,
    mut ui_events: EventReader<UiEvent>,
    poi_query: Query<(Entity, &Poi)>,
) {
    for event in ui_events.read() {
        let mut count = 0;
        match event {
            UiEvent::UnloadMarker(marker_id) => {
                for (entity, poi) in &poi_query {
                    if poi.0 == *marker_id {
                        commands.entity(entity).despawn_recursive();
                        count += 1;
                    }
                }
            }
            UiEvent::UnloadAllMarkers => {
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

#[derive(Resource, Clone, Deref, DerefMut, Debug, Default)]
pub struct LoadedMarkers(pub Vec<String>);

#[derive(Resource, Clone, Deref, Debug)]
pub struct MarkerTree(pub crate::parser::MarkerTree);
