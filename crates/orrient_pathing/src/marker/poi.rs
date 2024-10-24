use bevy::prelude::*;
use orrient_core::prelude::*;

use super::EnabledMarkers;
use crate::events::MarkerEvent;
use crate::parser::model::Behavior;
use crate::parser::MarkerPacks;
use crate::prelude::FullMarkerId;

use bevy_mod_billboard::plugin::BillboardPlugin;
use bevy_mod_billboard::BillboardMeshHandle;
use bevy_mod_billboard::BillboardTextBundle;
use bevy_mod_billboard::BillboardTextureBundle;
use bevy_mod_billboard::BillboardTextureHandle;

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, asset_server: Res<AssetServer>) {
    commands.insert_resource(PoiQuad(meshes.add(Rectangle::from_size(Vec2::splat(2.0)))));
    commands.insert_resource(MissingIcon(asset_server.load("missing.png")));
}

#[derive(Resource)]
struct MissingIcon(Handle<Image>);

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

#[derive(Event, Clone, Debug)]
struct SpawnPoiEvent(FullMarkerId);

#[derive(Event, Clone, Debug)]
struct DespawnPoiEvent(FullMarkerId);

fn marker_event_system(
    mut marker_events: EventReader<MarkerEvent>,
    mut spawn_events: EventWriter<SpawnPoiEvent>,
    mut despawn_events: EventWriter<DespawnPoiEvent>,
    enabled_markers: Res<EnabledMarkers>,
) {
    for event in marker_events.read() {
        match event {
            MarkerEvent::Enable(full_id) => {
                println!("spawn: {:?}", full_id);
                spawn_events.send(SpawnPoiEvent(full_id.clone()));
            }
            MarkerEvent::Disable(full_id) => {
                despawn_events.send(DespawnPoiEvent(full_id.clone()));
            }
            MarkerEvent::DisableAll => {
                despawn_events.send_batch(
                    enabled_markers
                        .iter()
                        .map(ToOwned::to_owned)
                        .map(DespawnPoiEvent),
                );
            }
        }
    }
}

fn spawn_pois_system(
    mut commands: Commands,
    mut events: EventReader<SpawnPoiEvent>,
    assets: Res<PoiQuad>,
    packs: Res<MarkerPacks>,
    map_id: Res<MapId>,
    missing_icon: Res<MissingIcon>,
) {
    let mut count = 0;
    for event in events.read() {
        let SpawnPoiEvent(full_id) = event;

        let Some(pack) = &packs.get(&full_id.pack_id) else {
            continue;
        };

        let Some(marker) = pack
            .find_by_name(full_id.marker_name.clone())
            .and_then(|node_id| pack.get(node_id))
            .map(|node| node.data())
        else {
            continue;
        };

        for poi in marker
            .pois
            .iter()
            .filter(|poi| poi.map_id == Some(map_id.0))
        {
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
                .or_else(|| marker.icon_file.clone())
                .map(|icon_path| icon_path.into_string())
                .and_then(|path| pack.get_image(&path));

            let mut builder = commands.spawn(TransformBundle::from_transform(
                Transform::from_translation(pos),
            ));

            if let Some(icon) = icon {
                builder.with_children(|parent| {
                    parent.spawn(BillboardTextureBundle {
                        mesh: BillboardMeshHandle(assets.0.clone()),
                        texture: BillboardTextureHandle(icon),
                        ..default()
                    });
                });
            } else {
                warn!("No icon for {:?}", full_id);
                builder.with_children(|parent| {
                    parent.spawn(BillboardTextureBundle {
                        mesh: BillboardMeshHandle(assets.0.clone()),
                        texture: BillboardTextureHandle(missing_icon.0.clone()),
                        transform: Transform::from_scale(Vec3::splat(0.25)),
                        ..default()
                    });
                    let display_name = marker.label.to_string();
                    parent.spawn(BillboardTextBundle {
                        text: Text::from_section(
                            display_name,
                            TextStyle {
                                font_size: 32.,
                                ..default()
                            },
                        ),
                        transform: Transform::from_scale(Vec3::splat(0.01))
                            .with_translation(Vec3::Y * -0.5),
                        ..default()
                    });
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
    mut events: EventReader<DespawnPoiEvent>,
) {
    for full_id in events.read().map(|event| &event.0) {
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
}

fn map_exit_system(mut commands: Commands, poi_query: Query<Entity, With<PoiMarker>>) {
    for entity in &poi_query {
        commands.entity(entity).despawn_recursive();
    }
}

fn map_enter_system(enabled_markers: Res<EnabledMarkers>, mut events: EventWriter<SpawnPoiEvent>) {
    events.send_batch(
        enabled_markers
            .iter()
            .map(ToOwned::to_owned)
            .map(SpawnPoiEvent),
    );
}

pub(super) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnPoiEvent>();
        app.add_event::<DespawnPoiEvent>();

        app.add_plugins(BillboardPlugin);

        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            disappear_nearby_system
                .run_if(in_state(GameState::InGame))
                .run_if(on_event::<WorldEvent>()),
        );
        app.add_systems(
            Update,
            spawn_pois_system
                .run_if(in_state(GameState::InGame))
                .run_if(on_event::<SpawnPoiEvent>()),
        );

        app.add_systems(
            Update,
            despawn_pois_system
                .run_if(in_state(GameState::InGame))
                .run_if(on_event::<DespawnPoiEvent>()),
        );

        app.add_systems(
            Update,
            marker_event_system
                .run_if(in_state(GameState::InGame))
                .run_if(on_event::<MarkerEvent>()),
        );

        app.add_systems(OnEnter(GameState::ChangingMaps), map_exit_system);
        app.add_systems(OnEnter(GameState::InGame), map_enter_system);
    }
}
