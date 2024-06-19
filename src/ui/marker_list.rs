use bevy::{
    color::palettes::basic,
    ecs::system::EntityCommands,
    input::mouse::{self, MouseWheel},
    prelude::*,
};

use crate::marker::{Category, Markers};

use super::UIElement;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, scroll);
    }
}

#[derive(Component, Default)]
struct ScrollBox {
    position: f32,
}

#[derive(Component, Default)]
pub struct MarkerList(pub Markers);

impl UIElement for MarkerList {
    fn build(&self, entity: &mut EntityCommands) {
        entity.insert(ScrollBox::default());
        entity
            .insert(NodeBundle {
                style: Style {
                    width: Val::Px(800.),
                    height: Val::Px(600.),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: basic::RED.into(),
                ..default()
            })
            .with_children(|parent| {
                for category in self.0.values() {
                    add_category(0, category, parent);
                }
            });
    }
}

fn add_category(indent: u8, category: &Category, parent: &mut ChildBuilder) {
    CategoryListItem {
        indent_level: indent,
        category: category.clone(),
    }
    .as_child(parent);
    for subcat in category.subcategories.values() {
        add_category(indent + 1, subcat, parent);
    }
}

#[derive(Component)]
struct CategoryListItem {
    indent_level: u8,
    category: Category,
}

impl UIElement for CategoryListItem {
    fn build(&self, entity: &mut EntityCommands) {
        entity.insert(TextBundle {
            style: Style {
                left: Val::Px(self.indent_level as f32 * 8.),
                ..default()
            },
            text: Text::from_section(
                self.category.name.clone(),
                TextStyle {
                    font_size: 16.0,
                    ..default()
                },
            ),
            ..default()
        });
    }
}

fn scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query_scrollbox: Query<(&mut ScrollBox, &mut Style, &Parent, &Node)>,
    query_node: Query<&Node>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        for (mut scrolling_list, mut style, parent, list_node) in &mut query_scrollbox {
            println!("scroll");
            let items_height = list_node.size().y;
            let container_height = query_node.get(parent.get()).unwrap().size().y;
            let max_scroll = (items_height - container_height).max(0.);
            let dy = match mouse_wheel_event.unit {
                mouse::MouseScrollUnit::Line => mouse_wheel_event.y * 20.,
                mouse::MouseScrollUnit::Pixel => mouse_wheel_event.y,
            };

            scrolling_list.position += dy;
            scrolling_list.position = scrolling_list.position.clamp(-max_scroll, 0.);
            style.top = Val::Px(scrolling_list.position);
        }
    }
}
