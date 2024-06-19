mod icon;
mod marker_list;

use bevy::{prelude::*, window::PrimaryWindow};
use icon::MainIcon;

use crate::{marker::MarkerSet, OrrientEvent};

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

fn setup(world: &mut World) {
    let parent = world.spawn(NodeBundle::default()).id();
    UIElement::spawn(MainCanvas, world, parent);
}

trait UIElement: Component + Sized {
    fn build(&self, world: &mut World) -> Entity;

    fn spawn(self, world: &mut World, parent: Entity) {
        let child = self.build(world);
        world.entity_mut(child).set_parent(parent).insert(self);
    }
}

#[derive(Component, Default)]
struct MainCanvas;

impl UIElement for MainCanvas {
    fn build(&self, world: &mut World) -> Entity {
        let entity = world
            .spawn((
                Name::new("MainCanvas"),
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        ..default()
                    },
                    visibility: Visibility::Visible,
                    ..default()
                },
            ))
            .id();

        UIElement::spawn(icon::MainIcon, world, entity);
        let markers = world.resource::<MarkerSet>();
        UIElement::spawn(marker_list::MarkerList(markers.clone()), world, entity);

        entity
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
