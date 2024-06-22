use bevy::prelude::*;

use crate::OrrientEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            update_position_system.run_if(on_event::<OrrientEvent>()),
        );
        // app.add_systems(Update, position);
        app.add_systems(Update, save_pos_system);
    }
}

#[derive(Component)]
struct Player;

fn setup(mut commands: Commands) {
    let mut entity = commands.spawn_empty();
    entity.insert(Player);
    entity.insert(TransformBundle::default());
}

fn update_position_system(
    mut events: EventReader<OrrientEvent>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut transform = query.single_mut();
    for event in events.read() {
        if let OrrientEvent::PlayerPositon(position) = event {
            transform.translation = *position;
        }
    }
}

fn position(
    mut gizmos: Gizmos,
    player: Query<&Transform, With<Player>>,
    saved: Query<&Transform, With<SavedPosition>>,
) {
    let position = Vec3::new(0., 120., 0.);
    gizmos.sphere(position, Quat::default(), 1.0, Color::RED);

    if let Ok(saved_pos) = saved.get_single() {
        let pos = saved_pos.translation;
        gizmos.sphere(pos, Quat::default(), 1.0, Color::FUCHSIA);
    }

    let player = player.single().translation;
    gizmos.arrow(player, player + Vec3::X, Color::RED);
    gizmos.arrow(player, player + Vec3::Y, Color::GREEN);
    gizmos.arrow(player, player + Vec3::Z, Color::BLUE);
}

#[derive(Component)]
struct SavedPosition;

fn save_pos_system(
    mut commands: Commands,
    mut events: EventReader<OrrientEvent>,
    player: Query<&Transform, With<Player>>,
    mut saved: Query<&mut Transform, (With<SavedPosition>, Without<Player>)>,
) {
    for event in events.read() {
        if let OrrientEvent::SavePosition = event {
            let player = player.single().translation;
            if let Ok(mut pos) = saved.get_single_mut() {
                pos.translation = player;
                println!("position updated");
            } else {
                commands.spawn((SavedPosition, Transform::from_translation(player)));
                println!("new position saved");
            }
        }
    }
}
