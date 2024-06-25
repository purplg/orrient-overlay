mod components;
mod routes;

use std::f32::consts::PI;

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
    },
};
use bevy_lunex::prelude::*;

use crate::marker::MarkerSet;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiPlugin);

        app.add_plugins(components::Plugin);
        app.add_plugins(routes::Plugin);

        app.add_systems(Startup, setup);
        app.add_systems(Startup, spawn_debug_mesh);
        app.add_systems(Update, move_debug_mesh);
        app.add_systems(Update, load_markers.run_if(resource_added::<MarkerSet>));
    }
}

#[derive(Component)]
pub struct UiRoot;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Render the 3d camera as a texture
    let size = Extent3d {
        width: 2560,
        height: 1440,
        ..default()
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    image.resize(size);
    let render_image = asset_server.add(image);
    commands.spawn(Camera3dBundle {
        camera: Camera {
            // The UI camera includes this 3D view as an image.
            target: render_image.clone().into(),
            // Thus we must lower the order so this camera is rendered
            // before the UI camera.
            order: -1,
            clear_color: ClearColorConfig::Custom(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            ..default()
        },
        projection: Projection::Perspective(PerspectiveProjection {
            fov: 70.32_f32.to_radians(),
            ..default()
        }),
        ..default()
    });

    // Spawn 2D Camera
    commands.spawn((
        Name::new("UI Camera"),
        MainUi,
        InheritedVisibility::default(),
        Camera2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 1000.0),
            ..default()
        },
    ));

    commands
        .spawn((
            Name::new("UI"),
            UiTreeBundle::<MainUi>::from(UiTree::new("MainUI")),
            MovableByCamera,
        ))
        .with_children(|ui| {
            let root = UiLink::<MainUi>::path("Root");

            ui.spawn((
                Name::new("Camera3d Image"),
                root.add("Camera3d Image"),
                UiLayout::window_full() //
                    .pack::<Base>(),
                UiImage2dBundle::from(render_image),
                Pickable::IGNORE,
            ));

            ui.spawn((
                root,
                UiRoot,
                UiLayout::solid() //
                    .size((16., 9.))
                    .pack::<Base>(),
            ));
        });
}

fn load_markers(mut commands: Commands) {
    commands.spawn(routes::MarkerList);
}

#[derive(Component)]
struct DebugMesh;

fn move_debug_mesh(mut query: Query<&mut Transform, With<DebugMesh>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.translation.x = time.elapsed_seconds().cos();
        transform.translation.y = time.elapsed_seconds().sin();
        transform.translation.z = time.elapsed_seconds().sin() - 10.;
        transform.rotation = Quat::from_rotation_y(time.elapsed_seconds() % PI * 2.)
    }
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

fn spawn_debug_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    commands.spawn((
        DebugMesh,
        PbrBundle {
            mesh: meshes.add(Sphere::default().mesh().ico(5).unwrap()),
            material: debug_material.clone(),
            transform: Transform::from_xyz(0.0, 0.0, -10.0),
            ..default()
        },
    ));
}
