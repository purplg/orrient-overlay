use bevy::prelude::*;
use bevy_lunex::prelude::*;

use crate::OrrientEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiGenericPlugin::<List>::new());
        app.add_systems(Update, build_list.before(UiSystems::Compute));
        app.add_systems(Update, scroll);
        app.add_systems(Update, select);
    }
}

#[derive(Component, Clone)]
pub struct List {
    items: Vec<ListItem>,
}

impl List {
    pub fn new<T: Into<ListItem>>(items: impl Iterator<Item = T>) -> Self {
        Self {
            items: items.into_iter().map(Into::into).collect(),
        }
    }
}

impl<'a> Into<ListItem> for marker::MarkerEntry<'a> {
    fn into(self) -> ListItem {
        ListItem {
            id: self.id.clone(),
            long_id: self.path.join("."),
            text: self.marker.label.clone(),
            kind: ListKind::Entry,
            indent_level: self.path.len(),
        }
    }
}

fn build_list(mut commands: Commands, query: Query<(Entity, &List), Added<List>>) {
    for (entity, list) in &query {
        commands
            .entity(entity)
            .insert((
                UiTreeBundle::<List>::from(UiTree::new("List")), //
            ))
            .with_children(|ui| {
                let root = UiLink::<List>::path("Root");

                let base = ui
                    .spawn((
                        root.clone(),
                        UiLayout::window_full().pack::<Base>(),
                        // Some interactivity stuff so we can capture
                        // scroll events to scroll the list of entries
                        UiZoneBundle::default(),
                        UiScrollEmitter::SELF,
                    ))
                    .id();

                let gap = 0.0;
                let size = 32.0;
                for (idx, item) in list.items.iter().enumerate() {
                    let root = root.add(idx.to_string());
                    // The text within the list entry.
                    let entity = ui
                        .spawn((
                            root.add("Text"),
                            UiLayout::window() //
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
                            UiClickEmitter::SELF,
                            UiScrollEmitter::new(base),
                        ))
                        .id();

                    // The base layout for a single list entry.
                    ui.spawn((
                        root,
                        UiLayout::window() //
                            .width(Rl(100.))
                            .height(size)
                            .x(item.indent_level as f32 * 8.)
                            .y(Ab(idx as f32 * (gap + size)))
                            .pack::<Base>(),
                        // Some interactivity stuff so we can capture
                        // click events to select entries
                        UiZoneBundle::default(),
                        UiClickEmitter::new(entity),
                        UiScrollEmitter::new(base),
                    ));
                }
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
    pub id: String,
    pub long_id: String,
    pub text: String,
    pub kind: ListKind,
    indent_level: usize,
}

impl ListItem {
    pub fn entry(id: String, long_id: String, text: String, indent_level: usize) -> Self {
        Self {
            id,
            long_id,
            text,
            kind: ListKind::Category,
            indent_level,
        }
    }
}

fn scroll(
    mut events: EventReader<UiScrollEvent>, //
    mut query_list: Query<&mut UiLayout>,
) {
    for event in events.read() {
        if let Ok(mut list) = query_list.get_mut(event.target) {
            let new_y = list.layout.expect_window().pos.get_y() + Rl(event.delta.y);
            list.layout.expect_window_mut().set_y(new_y);
        }
    }
}

fn select(
    mut events: EventReader<UiClickEvent>,
    mut orrient_events: EventWriter<OrrientEvent>,
    query_items: Query<&ListItem>,
) {
    for event in events.read() {
        if let Ok(item) = query_items.get(event.target) {
            orrient_events.send(OrrientEvent::LoadMarker(item.long_id.clone()));
        }
    }
}
