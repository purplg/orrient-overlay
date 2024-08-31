use bevy::prelude::*;

use crate::events::WorldEvent;

#[derive(Component)]
pub struct Player;

fn setup(mut commands: Commands) {
    let mut entity = commands.spawn_empty();
    entity.insert(Player);
    entity.insert(TransformBundle::default());
}

fn update_position_system(
    mut events: EventReader<WorldEvent>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut transform = query.single_mut();
    for event in events.read() {
        if let WorldEvent::PlayerPositon(position) = event {
            transform.translation = *position;
        }
    }
}

#[derive(Component)]
struct SavedPosition;

fn save_pos_system(
    mut commands: Commands,
    mut events: EventReader<WorldEvent>,
    player: Query<&Transform, With<Player>>,
    mut saved: Query<&mut Transform, (With<SavedPosition>, Without<Player>)>,
) {
    for event in events.read() {
        if let WorldEvent::SavePosition = event {
            let player = player.single().translation;
            if let Ok(mut pos) = saved.get_single_mut() {
                pos.translation = player;
                info!("position updated");
            } else {
                commands.spawn((SavedPosition, Transform::from_translation(player)));
                info!("new position saved");
            }
        }
    }
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            update_position_system.run_if(on_event::<WorldEvent>()),
        );
        // app.add_systems(Update, position);
        app.add_systems(Update, save_pos_system);
    }
}
