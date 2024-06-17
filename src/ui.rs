use bevy::{prelude::*, window::PrimaryWindow};

use crate::link::MumbleLinkEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            toggle_hittest_system.run_if(on_event::<MumbleLinkEvent>()),
        );
    }
}

fn toggle_hittest_system(
    mut events: EventReader<MumbleLinkEvent>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
    for event in events.read() {
        if let MumbleLinkEvent::Toggle = event {
            let mut window = window.single_mut();
            window.cursor.hit_test = !window.cursor.hit_test;
            window.decorations = window.cursor.hit_test;
            println!("hittest: {:?}", window.cursor.hit_test);
        }
    }
}
