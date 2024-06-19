mod icon;
mod marker_list;

use bevy::{ecs::system::EntityCommands, prelude::*, window::PrimaryWindow};

use crate::{marker::Markers, OrrientEvent};

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            toggle_hittest_system.run_if(on_event::<OrrientEvent>()),
        );
        app.add_plugins(icon::Plugin);
        app.add_plugins(marker_list::Plugin);
    }
}

fn setup(mut commands: Commands, markers: Res<Markers>) {
    MainCanvas.insert(&mut commands.spawn_empty());
    marker_list::MarkerList(markers.clone()).insert(&mut commands.spawn_empty());
}

trait UIElement: Component + Sized {
    fn insert(self, entity: &mut EntityCommands) {
        self.build(entity);
        entity.insert(self);
    }

    fn build(&self, entity: &mut EntityCommands);

    fn as_child(self, entity: &mut ChildBuilder) {
        let mut entity = entity.spawn_empty();
        self.build(&mut entity);
        entity.insert(self);
    }
}

#[derive(Component, Default)]
struct MainCanvas;

impl UIElement for MainCanvas {
    fn build(&self, entity: &mut EntityCommands) {
        entity
            .insert(NodeBundle {
                style: Style {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    ..default()
                },
                visibility: Visibility::Visible,
                ..default()
            })
            .with_children(|parent| icon::MainIcon.as_child(parent));
    }
}

fn toggle_hittest_system(
    mut events: EventReader<OrrientEvent>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut ui: Query<&mut Visibility, With<MainCanvas>>,
) {
    for event in events.read() {
        if let OrrientEvent::ToggleUI = event {
            let mut window = window.single_mut();
            window.cursor.hit_test = !window.cursor.hit_test;

            window.decorations = window.cursor.hit_test;
            *ui.single_mut() = if window.cursor.hit_test {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };

            println!(
                "UI {}",
                if window.cursor.hit_test {
                    "enabled"
                } else {
                    "disabled"
                }
            );
        }
    }
}
