use bevy::{
    input::mouse::{self, MouseWheel},
    prelude::*,
};

use crate::marker::{Category, Markers};

use super::UIElement;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ButtonEvent>();
        app.add_systems(Update, collapse_button_system);
        app.add_systems(Update, toggle_expand_system);
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
    fn build(&self, world: &mut World) -> Entity {
        let parent = world
            .spawn((
                ScrollBox::default(),
                Name::new("MarkerList"),
                NodeBundle {
                    style: Style {
                        width: Val::Px(800.),
                        height: Val::Px(600.),
                        justify_content: JustifyContent::Start,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color: Color::RED.into(),
                    ..default()
                },
            ))
            .id();

        for category in self.0.values() {
            UIElement::spawn(MarkerListItem(category.clone()), world, parent);
        }

        parent
    }
}

#[derive(Component)]
struct MarkerListItem(Category);

impl UIElement for MarkerListItem {
    fn build(&self, world: &mut World) -> Entity {
        let has_children = self.0.subcategories.len() > 0;
        let parent = world
            .spawn(NodeBundle {
                style: Style {
                    left: if has_children { Val::Px(8.) } else { Val::Auto },
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            })
            .id();

        let content = world
            .spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
                ..default()
            })
            .set_parent(parent)
            .id();

        if has_children {
            UIElement::spawn(MarkerCollapseButton(self.0.id.clone()), world, content);
        }

        UIElement::spawn(MarkerText(self.0.name.clone()), world, content);

        let subitems = world
            .spawn((
                SubCategories(self.0.id.clone()),
                NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ..default()
                },
            ))
            .set_parent(parent)
            .id();

        for category in self.0.subcategories.values() {
            UIElement::spawn(MarkerListItem(category.clone()), world, subitems);
        }

        parent
    }
}

#[derive(Component)]
struct SubCategories(String);

#[derive(Component)]
struct Collapsed;

#[derive(Component)]
struct MarkerText(String);

impl UIElement for MarkerText {
    fn build(&self, world: &mut World) -> Entity {
        world
            .spawn((
                Name::new(format!("MarkerListItem: {}", self.0)),
                TextBundle {
                    style: Style {
                        padding: UiRect {
                            left: Val::Px(16.),
                            top: Val::Px(16.),
                            ..default()
                        },
                        left: Val::Px(8.),
                        justify_content: JustifyContent::Start,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    text: Text::from_section(
                        self.0.clone(),
                        TextStyle {
                            font_size: 16.0,
                            ..default()
                        },
                    ),
                    ..default()
                },
            ))
            .id()
    }
}

#[derive(Component)]
struct MarkerCollapseButton(String);

impl UIElement for MarkerCollapseButton {
    fn build(&self, world: &mut World) -> Entity {
        world
            .spawn((
                Name::new("MarkerCollapseButton"),
                ButtonBundle {
                    style: Style {
                        width: Val::Px(16.),
                        height: Val::Px(16.),
                        ..default()
                    },
                    background_color: Color::PURPLE.into(),
                    ..default()
                },
            ))
            .id()
    }
}

#[derive(Event)]
struct ButtonEvent(String);

fn collapse_button_system(
    input: Res<ButtonInput<MouseButton>>,
    mut events: EventWriter<ButtonEvent>,
    query_buttons: Query<(&Interaction, &MarkerCollapseButton)>,
) {
    for (interaction, button) in &query_buttons {
        match *interaction {
            Interaction::Hovered => {}
            Interaction::Pressed => {
                if input.just_pressed(MouseButton::Left) {
                    events.send(ButtonEvent(button.0.clone()));
                }
            }
            Interaction::None => {}
        }
    }
}

fn toggle_expand_system(
    mut events: EventReader<ButtonEvent>,
    mut query_subcategories: Query<(&mut Visibility, &mut Style, &SubCategories)>,
) {
    for event in events.read() {
        for (mut visibility, mut style, subcat) in query_subcategories.iter_mut() {
            if subcat.0 == event.0 {
                match *visibility {
                    Visibility::Hidden => {
                        style.display = Display::DEFAULT;
                        *visibility = Visibility::Visible;
                    }
                    _ => {
                        *visibility = Visibility::Hidden;
                        style.display = Display::None;
                    }
                }
            }
        }
    }
}

fn scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query_scrollbox: Query<(&mut ScrollBox, &mut Style, &Parent, &Node)>,
    query_node: Query<&Node>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        for (mut scrolling_list, mut style, parent, list_node) in &mut query_scrollbox {
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
