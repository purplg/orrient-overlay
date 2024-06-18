use bevy::prelude::*;

use crate::OrrientEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, camera_system);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        projection: Projection::Perspective(PerspectiveProjection {
            fov: 70.32_f32.to_radians(),
            ..default()
        }),
        ..default()
    });
}

fn camera_system(
    mut events: EventReader<OrrientEvent>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera3d>>,
) {
    for event in events.read() {
        match event {
            OrrientEvent::CameraUpdate {
                position,
                facing,
                fov,
            } => {
                let (mut transform, projection) = camera.single_mut();
                transform.translation = *position;

                transform.rotation = Quat::IDENTITY;
                transform.rotate_x(facing.y.asin());
                transform.rotate_y(-facing.x.atan2(facing.z));

                if let Projection::Perspective(perspective) = projection.into_inner() {
                    perspective.fov = *fov;
                }
            }
            _ => (),
        }
    }
}
