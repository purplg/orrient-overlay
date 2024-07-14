use bevy::{
    color::palettes,
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat},
    },
    utils::HashMap,
};
use itertools::Itertools;

use crate::{parser::prelude::*, UiEvent};

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TrailMeshes>();
        app.add_plugins(MaterialPlugin::<TrailMaterial>::default());
        app.add_systems(Startup, load_marker_assets);
        app.add_systems(Update, trail_event.run_if(on_event::<UiEvent>()));
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
struct TrailMeshes(HashMap<FullMarkerId, Vec<Entity>>);

#[derive(Component)]
struct TrailMesh;

#[derive(Resource)]
pub struct DebugMarkerAssets {
    pub mesh: Handle<Mesh>,
    pub image_quad: Handle<Mesh>,
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
    let image_quad = meshes.add(Rectangle::from_size(Vec2::splat(2.0)));
    commands.insert_resource(DebugMarkerAssets {
        mesh,
        image_quad,
        trails_mesh: None,
        trail_material: materials.add(StandardMaterial {
            base_color_texture: Some(images.add(uv_debug_texture())),
            base_color: palettes::tailwind::BLUE_500.into(),
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
        poi_material: materials.add(StandardMaterial {
            base_color_texture: Some(images.add(uv_debug_texture())),
            base_color: palettes::tailwind::RED_500.into(),
            ..default()
        }),
    })
}

const TRAIL_WIDTH: f32 = 0.5;

#[derive(Clone, Copy)]
struct OrientedPoint {
    position: Vec3,
    forward: Vec3,
    distance: f32,
}

fn create_trail_mesh(path: impl Iterator<Item = Vec3>) -> Mesh {
    let mut indices: Vec<u32> = vec![];
    let mut positions: Vec<Vec3> = vec![];
    let mut uvs: Vec<Vec2> = vec![];
    let mut normals: Vec<Vec3> = vec![];

    let mut distance: f32 = 0.0;
    let points = path.tuple_windows().map(|(prev_pos, next_pos)| {
        let forward = *Dir3::new_unchecked((next_pos - prev_pos).normalize());
        distance += prev_pos.distance(next_pos);
        OrientedPoint {
            position: next_pos,
            forward,
            distance,
        }
    });

    for (prev_pos, next_pos) in points.tuple_windows() {
        let prev_left_vertex = positions.len() as u32;
        positions.push(prev_pos.position + prev_pos.forward.cross(Vec3::NEG_Y) * TRAIL_WIDTH);
        uvs.push(Vec2::new(0.0, prev_pos.distance));
        normals.push(Vec3::Z);

        let prev_right_vertex = positions.len() as u32;
        positions.push(prev_pos.position + prev_pos.forward.cross(Vec3::Y) * TRAIL_WIDTH);
        uvs.push(Vec2::new(1.0, prev_pos.distance));
        normals.push(Vec3::Z);

        let next_left_vertex = positions.len() as u32;
        positions.push(next_pos.position + next_pos.forward.cross(Vec3::NEG_Y) * TRAIL_WIDTH);
        uvs.push(Vec2::new(0.0, next_pos.distance));
        normals.push(Vec3::Z);

        let next_right_vertex = positions.len() as u32;
        positions.push(next_pos.position + next_pos.forward.cross(Vec3::Y) * TRAIL_WIDTH);
        uvs.push(Vec2::new(1.0, next_pos.distance));
        normals.push(Vec3::Z);

        indices.push(prev_left_vertex);
        indices.push(prev_right_vertex);
        indices.push(next_right_vertex);
        indices.push(next_right_vertex);
        indices.push(next_left_vertex);
        indices.push(prev_left_vertex);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(indices))
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct TrailMaterial {
    #[uniform(0)]
    color: LinearRgba,
    #[texture(1)]
    #[sampler(2)]
    color_texture: Option<Handle<Image>>,
    alpha_mode: AlphaMode,
    #[uniform(3)]
    speed: f32,
}

impl Material for TrailMaterial {
    fn fragment_shader() -> ShaderRef {
        "trail.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

fn trail_event(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut events: EventReader<UiEvent>,
    mut trail_meshes: ResMut<TrailMeshes>,
    mut trail_materials: ResMut<Assets<TrailMaterial>>,
    packs: Res<MarkerPacks>,
) {
    for event in events.read() {
        match event {
            UiEvent::UnloadMarker(full_id) => {
                if let Some(entities) = trail_meshes.remove(full_id) {
                    for entity in entities {
                        info!("Unloading trail: {:?}", full_id);
                        commands.entity(entity).despawn_recursive();
                    }
                }
            }
            UiEvent::UnloadAllMarkers => {
                for (trail_id, entities) in trail_meshes.drain() {
                    for entity in entities {
                        info!("Unloading trail: {:?}", trail_id);
                        commands.entity(entity).despawn_recursive();
                    }
                }
            }

            UiEvent::LoadMarker(full_id) => {
                let Some(pack) = &packs.get(&full_id.pack_id) else {
                    warn!("Pack ID not found: {}", full_id.pack_id);
                    continue;
                };

                let Some(trails) = pack.get_trails(&full_id.marker_id) else {
                    warn!("Trail not found for marker_id: {full_id}");
                    continue;
                };

                debug!("Loading trails for {}...", full_id);

                for trail in trails.iter() {
                    let iter = trail.path.iter().map(|path| Vec3 {
                        x: path.x,
                        y: path.y,
                        z: -path.z,
                    });

                    let Some(texture) = pack.get_image(&trail.texture_file) else {
                        warn!("Could not find texture {}", trail.texture_file);
                        continue;
                    };

                    debug!("Trail texture: {:?}", trail.texture_file);

                    let material = trail_materials.add(TrailMaterial {
                        color: palettes::tailwind::ZINC_500.into(),
                        color_texture: Some(texture),
                        alpha_mode: AlphaMode::Blend,
                        speed: 1.0,
                    });

                    let mesh = create_trail_mesh(iter);

                    let entity = commands
                        .spawn((
                            TrailMesh,
                            MaterialMeshBundle {
                                mesh: meshes.add(mesh),
                                material,
                                ..default()
                            },
                        ))
                        .id();

                    if let Some(entities) = trail_meshes.get_mut(full_id) {
                        entities.push(entity);
                    } else {
                        trail_meshes.insert(full_id.clone(), vec![entity]);
                    }
                }
                info!("Trail {} loaded.", full_id);
            }
            UiEvent::ToggleUI => {}
            UiEvent::ShowMarkerBrowser => {}
        }
    }
}
