use bevy::color::palettes::basic;
use bevy::prelude::*;

use crate::link::MumbleLinkEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, position);
        app.add_systems(Update, save_pos_system);
    }
}

fn position(
    mut gizmos: Gizmos,
    mumbledata: Res<MumbleData>,
    query: Query<&Transform, With<SavedPosition>>,
) {
    let position = Vec3::new(0., 120., 0.);
    gizmos.sphere(position, Quat::default(), 1.0, basic::RED);

    if let Ok(saved_pos) = query.get_single() {
        let pos = saved_pos.translation;
        gizmos.sphere(pos, Quat::default(), 1.0, basic::FUCHSIA);
    }

    if let Some(data) = &mumbledata.0 {
        let player = Vec3::new(
            data.avatar.position[0],
            data.avatar.position[1],
            -data.avatar.position[2],
        );
        gizmos.arrow(player, player + Vec3::X, basic::RED);
        gizmos.arrow(player, player + Vec3::Y, basic::GREEN);
        gizmos.arrow(player, player + Vec3::Z, basic::BLUE);
    }
}

#[derive(Component)]
struct SavedPosition;

fn save_pos_system(
    mut commands: Commands,
    mut events: EventReader<MumbleLinkEvent>,
    mumbledata: Res<MumbleData>,
    mut query: Query<&mut Transform, With<SavedPosition>>,
) {
    for event in events.read() {
        if let MumbleLinkEvent::Save = event {
            if let Some(data) = &mumbledata.0 {
                if let Ok(mut pos) = query.get_single_mut() {
                    pos.translation = Vec3::new(
                        data.avatar.position[0],
                        data.avatar.position[1],
                        -data.avatar.position[2],
                    );
                    println!("position updated");
                } else {
                    commands.spawn((
                        SavedPosition,
                        Transform::from_xyz(
                            data.avatar.position[0],
                            data.avatar.position[1],
                            -data.avatar.position[2],
                        ),
                    ));
                    println!("new position saved");
                }
            }
        }
    }
}
