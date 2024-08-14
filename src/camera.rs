use crate::prelude::*;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::NONE));
        app.add_systems(Update, camera_system.run_if(in_state(AppState::Running)));
    }
}

fn camera_system(
    mut events: EventReader<WorldEvent>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera3d>>,
) {
    for event in events.read() {
        if let WorldEvent::CameraUpdate {
            position,
            facing,
            fov,
        } = event
        {
            let (mut transform, projection) = camera.single_mut();
            transform.translation = *position;

            transform.rotation = Quat::IDENTITY;
            transform.rotate_x(facing.y.asin());
            transform.rotate_y(-facing.x.atan2(facing.z));

            if let Projection::Perspective(perspective) = projection.into_inner() {
                perspective.fov = *fov;
            }
        }
    }
}
