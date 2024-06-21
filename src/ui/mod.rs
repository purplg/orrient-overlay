mod icon;
mod marker_list;

use bevy::{prelude::*, window::PrimaryWindow};

use crate::{marker::MarkerSet, OrrientEvent};

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ReloadUI>();

        app.add_systems(Startup, load_ui);
        app.add_systems(Update, load_ui.run_if(on_event::<ReloadUI>()));

        app.add_systems(
            Update,
            toggle_hittest_system.run_if(on_event::<OrrientEvent>()),
        );
        app.add_plugins(icon::Plugin);
        app.add_plugins(marker_list::Plugin);
    }
}

#[derive(Event)]
struct ReloadUI;

fn load_ui(world: &mut World) {
    if let Ok(maincanvas) = world
        .query_filtered::<Entity, With<MainCanvas>>()
        .get_single(world)
    {
        despawn_with_children_recursive(world, maincanvas);
    }
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
                        height: Val::Px(400.),
                        ..default()
                    },
                    visibility: Visibility::Visible,
                    ..default()
                },
            ))
            .id();

        UIElement::spawn(icon::MainIcon, world, entity);
        if let Some(markers) = world.get_resource::<MarkerSet>() {
            UIElement::spawn(
                marker_list::MarkerList(markers.markers.clone()),
                world,
                entity,
            );
        }

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
