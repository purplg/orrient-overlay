use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    utils::HashMap,
};
use bevy_mod_billboard::plugin::BillboardPlugin;
use smesh::smesh::{SMesh, VertexId};

use crate::{marker::MarkerTree, UiEvent};

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BillboardPlugin);
        app.init_resource::<TrailMeshes>();
        app.add_systems(Startup, load_marker_assets);
        app.add_systems(Update, trail_event.run_if(on_event::<UiEvent>()));
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
struct TrailMeshes(HashMap<String, Vec<Entity>>);

#[derive(Component)]
struct TrailMesh;

#[derive(Resource)]
pub struct DebugMarkerAssets {
    pub mesh: Handle<Mesh>,
    pub trails_mesh: Option<Handle<Mesh>>,
    pub trail_material: Handle<StandardMaterial>,
    pub poi_material: Handle<StandardMaterial>,
}

pub fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn load_marker_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Sphere::default().mesh().ico(5).unwrap());
    commands.insert_resource(DebugMarkerAssets {
        mesh,
        trails_mesh: None,
        trail_material: materials.add(StandardMaterial {
            base_color_texture: Some(images.add(uv_debug_texture())),
            base_color: Color::BLUE,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
        poi_material: materials.add(StandardMaterial {
            base_color_texture: Some(images.add(uv_debug_texture())),
            base_color: Color::RED,
            ..default()
        }),
    })
}

const TRAIL_WIDTH: f32 = 0.2;

fn create_trail_mesh(mut path: impl Iterator<Item = Vec3>) -> Option<Mesh> {
    let mut smesh = SMesh::new();
    let mut previous_pos: Vec3;
    let mut previous_left: VertexId;
    let mut previous_right: VertexId;

    if let (Some(prev_pos), Some(next_pos)) = (path.next(), path.next()) {
        let forward = *Direction3d::new_unchecked((next_pos - prev_pos).normalize());
        let prev_left_vertex =
            smesh.add_vertex(prev_pos + forward.cross(Vec3::NEG_Y) * TRAIL_WIDTH);
        let prev_right_vertex = smesh.add_vertex(prev_pos + forward.cross(Vec3::Y) * TRAIL_WIDTH);
        let next_left_vertex =
            smesh.add_vertex(next_pos + forward.cross(Vec3::NEG_Y) * TRAIL_WIDTH);
        let next_right_vertex = smesh.add_vertex(next_pos + forward.cross(Vec3::Y) * TRAIL_WIDTH);
        let _ = smesh.add_face(vec![
            prev_left_vertex,
            prev_right_vertex,
            next_right_vertex,
            next_left_vertex,
        ]);
        previous_pos = next_pos;
        previous_left = next_left_vertex;
        previous_right = next_right_vertex;
    } else {
        return None;
    }

    for next_pos in path {
        let forward = *Direction3d::new_unchecked((next_pos - previous_pos).normalize());

        let next_left = smesh.add_vertex(next_pos + forward.cross(Vec3::NEG_Y) * TRAIL_WIDTH);
        let next_right = smesh.add_vertex(next_pos + forward.cross(Vec3::Y) * TRAIL_WIDTH);
        let _ = smesh.add_face(vec![previous_left, previous_right, next_right, next_left]);
        previous_pos = next_pos;
        previous_left = next_left;
        previous_right = next_right;
    }

    Some(Mesh::from(smesh))
}

fn trail_event(
    mut commands: Commands,
    mut assets: ResMut<DebugMarkerAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut events: EventReader<UiEvent>,
    mut trail_meshes: ResMut<TrailMeshes>,
    markers: Res<MarkerTree>,
) {
    for event in events.read() {
        match event {
            UiEvent::UnloadMarker(trail_id) => {
                if let Some(entities) = trail_meshes.remove(trail_id) {
                    for entity in entities {
                        info!("Unloading trail: {:?}", trail_id);
                        commands.entity(entity).despawn_recursive();
                    }
                }
            }
            UiEvent::LoadMarker(trail_id) => {
                let Some(trails) = markers.0.get_trails(trail_id) else {
                    warn!("Trail not found for marker_id: {trail_id}");
                    return;
                };

                for trail in trails {
                    info!("Loading trail: {}...", trail_id);

                    let iter = trail.path.iter().map(|path| Vec3 {
                        x: path.x,
                        y: path.y,
                        z: path.z,
                    });

                    let handle = if let Some(trail_mesh) = create_trail_mesh(iter) {
                        let handle = meshes.add(trail_mesh);
                        assets.trails_mesh = Some(handle.clone());
                        handle
                    } else {
                        return;
                    };

                    let entity = commands
                        .spawn((
                            TrailMesh,
                            PbrBundle {
                                mesh: handle,
                                material: assets.trail_material.clone(),
                                ..default()
                            },
                        ))
                        .id();

                    if let Some(entities) = trail_meshes.get_mut(&trail_id.to_string()) {
                        entities.push(entity);
                    } else {
                        trail_meshes.insert(trail_id.to_string(), vec![entity]);
                    }
                    info!("Trail {} loaded.", trail_id);
                }
            }
            UiEvent::ToggleUI => {}
            UiEvent::ShowMarkerBrowser => {}
        }
    }
}
