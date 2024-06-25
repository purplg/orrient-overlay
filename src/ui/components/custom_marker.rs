use std::time::Duration;

use bevy::{prelude::*, sprite::Anchor, time::common_conditions::on_timer, window::PrimaryWindow};
use bevy_lunex::prelude::*;

use crate::OrrientEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiGenericPlugin::<Marker>::new());
        app.add_systems(Update, build_route.before(UiSystems::Compute));
        app.add_systems(Update, update_system);
    }
}

#[derive(Component)]
pub struct Marker;

fn build_route(mut commands: Commands, query: Query<Entity, Added<Marker>>) {
    for entity in &query {
        commands
            .entity(entity)
            .insert(UiTreeBundle::<Marker>::from(UiTree::new("CustomTest")))
            .with_children(|ui| {
                ui.spawn((
                    Marker,
                    UiLink::<Marker>::path("marker"),
                    UiText2dBundle {
                        text: Text::from_section(
                            "v",
                            TextStyle {
                                font_size: 32.,
                                ..default()
                            },
                        ),
                        ..default()
                    },
                ));
            });
    }
}

fn update_system(
    mut events: EventReader<OrrientEvent>,
    mut query_marker: Query<&mut UiLayout, With<Marker>>,
    query_camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
) {
    for event in events.read() {
        if let OrrientEvent::PlayerPositon(pos) = event {
            let (camera, camera_transform) = query_camera.single();
            if let Some(pos_2d) = camera.world_to_viewport(camera_transform, *pos) {
                for mut marker in &mut query_marker {
                    marker.layout.expect_window_mut().set_x(pos_2d.x);
                    marker.layout.expect_window_mut().set_y(pos_2d.y);
                }
            }
        }
    }
}
