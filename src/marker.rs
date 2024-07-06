use bevy::prelude::*;
use bevy_mod_billboard::BillboardTextBundle;
use marker::trail;

use crate::{player::Player, trail::DebugMarkerAssets, UiEvent, WorldEvent};

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
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
            load_trail_system.run_if(resource_exists_and_changed::<Marker>),
        );
        app.add_systems(
            Update,
            (load_pois_system, unload_pois_system).run_if(on_event::<UiEvent>()),
        );
    }
}

fn load_marker(mut commands: Commands, mut events: EventReader<UiEvent>, data: Res<MarkerTree>) {
    for event in events.read() {
        let UiEvent::LoadMarker(marker_id) = event else {
            return;
        };

        let Some(marker) = &data.get(marker_id) else {
            warn!("Marker ID not found: {}", marker_id);
            return;
        };

        commands.insert_resource(Marker((*marker).clone()));
    }
}

fn setup(mut commands: Commands) {
    let markers = match marker::read(&dirs::config_dir().unwrap().join("orrient").join("markers")) {
        Ok(markers) => markers,
        Err(err) => {
            println!("Error when loading markers: {:?}", err);
            return;
        }
    };

    commands.insert_resource(MarkerTree(markers));
}

#[derive(Resource)]
pub struct Trail(pub Vec<Vec3>);

fn load_trail_system(mut commands: Commands, marker: Res<Marker>) {
    let Some(trail) = &marker.trail_file else {
        info!("No trail for this marker category.");
        return;
    };

    let trail_path = dirs::config_dir()
        .unwrap()
        .join("orrient")
        .join("markers")
        .join(trail);

    let Ok(trail) = trail::from_file(trail_path.as_path()) else {
        error!("Error when loading trail file at: {:?}", trail_path);
        return;
    };

    commands.insert_resource(Trail(
        trail
            .coordinates
            .iter()
            .map(|coord| Vec3 {
                x: coord.x,
                y: coord.y,
                z: -coord.z,
            })
            .collect(),
    ));

    info!("Loaded trail with {} markers", trail.coordinates.len());
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
            return;
        };

        let pois = pois
            .iter()
            .map(|poi| Vec3 {
                x: poi.x,
                y: poi.y,
                z: -poi.z,
            })
            .collect::<Vec<_>>();

        for poi in &pois {
            let mut builder = commands.spawn((
                PbrBundle {
                    mesh: assets.mesh.clone(),
                    material: assets.poi_material.clone(),
                    transform: Transform::from_translation(*poi),
                    ..default()
                },
                Poi(marker_id.to_string()),
            ));

            debug!("Spawned POI at {}", poi);

            builder.with_children(|parent| {
                parent.spawn(BillboardTextBundle {
                    text: Text::from_section(
                        marker.label.clone(),
                        TextStyle {
                            font_size: 100.,
                            ..default()
                        },
                    ),
                    transform: Transform::from_scale(Vec3::ONE * 0.01)
                        .with_translation(Vec3::Y * 2.),
                    ..default()
                });
            });

            if let Some(marker::Behavior::ReappearDaily) = marker.behavior {
                builder.insert(DisappearNearby);
            } else if let Some(marker::Behavior::DisappearOnUse) = marker.behavior {
                builder.insert(DisappearNearby);
            }
        }

        info!("Loaded {} POIs.", pois.len());
    }
}

fn unload_pois_system(
    mut commands: Commands,
    mut ui_events: EventReader<UiEvent>,
    poi_query: Query<(Entity, &Poi)>,
) {
    for event in ui_events.read() {
        let UiEvent::UnloadMarker(marker_id) = event else {
            return;
        };

        let mut count = 0;
        for (entity, poi) in &poi_query {
            if poi.0 == *marker_id {
                commands.entity(entity).despawn_recursive();
                count += 1;
            }
        }
        if count > 0 {
            info!("Unloaded {} POIs.", count);
        }
    }
}

#[derive(Resource, Clone, Deref, Debug)]
pub struct Marker(pub marker::Marker);

#[derive(Resource, Clone, Deref, Debug)]
pub struct MarkerTree(pub marker::MarkerTree);
