use bevy::prelude::*;

use crate::link::MumbleLinkEvent;

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
    mut events: EventReader<MumbleLinkEvent>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera3d>>,
) {
    for event in events.read() {
        if let MumbleLinkEvent::Data(mumbledata) = event {
            let (mut transform, projection) = camera.single_mut();
            transform.translation = Vec3::new(
                mumbledata.camera.position[0],
                mumbledata.camera.position[1],
                -mumbledata.camera.position[2],
            );

            let Ok(forward) = Dir3::new(Vec3::new(
                mumbledata.camera.front[0],
                mumbledata.camera.front[1],
                mumbledata.camera.front[2],
            )) else {
                continue;
            };

            transform.rotation = Quat::IDENTITY;
            transform.rotate_x(forward.y.asin());
            transform.rotate_y(-forward.x.atan2(forward.z));

            if let Projection::Perspective(perspective) = projection.into_inner() {
                perspective.fov = mumbledata.identity.fov
            }
        }
    }
}
