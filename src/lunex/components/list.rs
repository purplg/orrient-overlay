use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy_lunex::prelude::*;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiGenericPlugin::<List>::new());
        app.add_plugins(UiDebugPlugin::<List>::new());
        app.add_systems(Update, build_component.before(UiSystems::Compute));
        app.add_systems(Update, scroll);
    }
}

#[derive(Component, Clone)]
pub struct List {
    items: Vec<ListItem>,
}

impl List {
    pub fn new<T: Into<ListItem>>(items: impl IntoIterator<Item = T>) -> Self {
        Self {
            items: items.into_iter().map(Into::into).collect(),
        }
    }
}

fn build_component(mut commands: Commands, query: Query<(Entity, &List), Added<List>>) {
    for (entity, list) in &query {
        commands
            .entity(entity)
            .insert((
                UiTreeBundle::<List>::from(UiTree::new("List")), //
            ))
            .with_children(|ui| {
                let root = UiLink::<List>::path("Root");

                let gap = 0.0;
                let size = 16.0;
                let mut offset = 0.0;
                let mut count = 0;
                for item in &list.items {
                    let link = root.add(format!("Item: {}", count));
                    ui.spawn((
                        link,
                        UiLayout::window() //
                            .width(100.)
                            .height(size)
                            .y(Ab(offset))
                            .x(item.indent_level as f32 * 8.)
                            .pack::<Base>(),
                        item.clone(),
                        UiText2dBundle {
                            text: Text::from_section(
                                item.text.clone(),
                                TextStyle {
                                    font_size: 100.,
                                    ..default()
                                },
                            ),
                            ..default()
                        },
                        // Some interactivity stuff so we can capture
                        // click events to select entries
                        // UiClickEmitter::SELF,
                        // Pickable::default(),
                    ));
                    offset += gap + size * 2.;
                    count += 1;
                }

                ui.spawn((
                    root,
                    UiLayout::window_full() //
                        .pack::<Base>(),
                    // Some interactivity stuff so we can capture scroll events
                    UiClickEmitter::SELF,
                    UiZoneBundle::default(),
                ));
            });
    }
}

#[derive(Clone)]
enum ListKind {
    Category,
    Entry,
    Separator,
}

#[derive(Component, Clone)]
pub struct ListItem {
    id: String,
    text: String,
    kind: ListKind,
    indent_level: u8,
}

impl ListItem {
    pub fn entry(id: String, text: String, indent_level: u8) -> Self {
        Self {
            id,
            text,
            kind: ListKind::Entry,
            indent_level,
        }
    }

    pub fn category(id: String, text: String, indent_level: u8) -> Self {
        Self {
            id,
            text,
            kind: ListKind::Category,
            indent_level,
        }
    }

    pub fn separator(id: String, text: String, indent_level: u8) -> Self {
        Self {
            id,
            text,
            kind: ListKind::Separator,
            indent_level,
        }
    }
}

fn scroll(
    mut events: EventReader<MouseWheel>, //
    mut query_list: Query<&mut UiLayout, With<List>>,
) {
    for event in events.read() {
        let amount = match event.unit {
            bevy::input::mouse::MouseScrollUnit::Pixel => Ab(event.y),
            bevy::input::mouse::MouseScrollUnit::Line => Ab(event.y * 20.),
        };
        for mut list in &mut query_list {
            let new_y = list.layout.expect_window().pos.get_y() + amount;
            list.layout.expect_window_mut().set_y(new_y);
        }
    }
}
