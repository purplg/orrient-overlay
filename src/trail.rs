use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};

use crate::marker::{POIs, Trail};

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_marker_assets);
        app.add_systems(Update, load_trail.run_if(resource_added::<Trail>));
        app.add_systems(Update, load_pois.run_if(resource_added::<POIs>));
    }
}

#[derive(Resource)]
pub struct DebugMarkerAssets {
    mesh: Handle<Mesh>,
    trail_material: Handle<StandardMaterial>,
    poi_material: Handle<StandardMaterial>,
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
        trail_material: materials.add(StandardMaterial {
            base_color_texture: Some(images.add(uv_debug_texture())),
            base_color: Color::BLUE,
            ..default()
        }),
        poi_material: materials.add(StandardMaterial {
            base_color_texture: Some(images.add(uv_debug_texture())),
            base_color: Color::RED,
            ..default()
        }),
    })
}

fn load_trail(mut commands: Commands, trail: Res<Trail>, assets: Res<DebugMarkerAssets>) {
    for pos in trail.0.iter() {
        commands.spawn(PbrBundle {
            mesh: assets.mesh.clone(),
            material: assets.trail_material.clone(),
            transform: Transform::from_translation(*pos).with_scale(Vec3::ONE * 0.5),
            ..default()
        });
    }
}

fn load_pois(mut commands: Commands, pois: Res<POIs>, assets: Res<DebugMarkerAssets>) {
    for pos in &pois.0 {
        commands.spawn(PbrBundle {
            mesh: assets.mesh.clone(),
            material: assets.poi_material.clone(),
            transform: Transform::from_translation(*pos),
            ..default()
        });
    }
}
