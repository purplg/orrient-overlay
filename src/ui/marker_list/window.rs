use bevy::{prelude::*, window::PrimaryWindow};
use sickle_ui::{
    ui_builder::{UiBuilder, UiBuilderExt as _},
    ui_style::generated::*,
    widgets::prelude::*,
};

use crate::{marker::MarkerTree, ui::OrrientMenuItem, UiEvent};

use super::{marker_button::UiMarkerButtonExt, tooltip::UiToolTipExt as _};

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CheckboxEvent>();
        app.add_event::<SetColumnEvent>();
        app.add_systems(Update, menu_interaction);
        app.add_systems(Update, set_column);
        app.add_systems(Update, toggle_show_ui);
        app.add_systems(Update, checkbox);
        app.add_systems(Update, checkbox_events);
        app.add_systems(
            Update,
            (remove_window, setup_window)
                .chain()
                .run_if(resource_exists_and_changed::<MarkerTree>),
        );
    }
}

#[derive(Event)]
pub(super) struct SetColumnEvent {
    pub column_id: usize,
    pub marker_id: Option<String>,
}

#[derive(Component)]
pub(super) struct MarkerWindow;

#[derive(Component)]
struct MarkerList;

#[derive(Component)]
pub(super) struct MarkerItem {
    pub id: String,
    pub tip: Option<String>,
    pub description: Option<String>,
}

pub trait UiMarkerWindowExt {
    fn marker_window(&mut self);
}

impl UiMarkerWindowExt for UiBuilder<'_, Entity> {
    fn marker_window(&mut self) {
        self.floating_panel(
            FloatingPanelConfig {
                title: Some("Markers".into()),
                ..default()
            },
            FloatingPanelLayout {
                size: (1920., 1080.).into(),
                position: Some((100., 100.).into()),
                ..default()
            },
            |parent| {
                parent.menu_bar(|parent| {
                    parent.menu(
                        MenuConfig {
                            name: "File".into(),
                            ..default()
                        },
                        |parent| {
                            parent
                                .menu_item(MenuItemConfig {
                                    name: "Open markers...".into(),
                                    ..default()
                                })
                                .insert(OrrientMenuItem(UiEvent::ShowMarkerBrowser));
                        },
                    );
                });

                parent
                    .spawn((
                        NodeBundle::default(),
                        MarkerList, //
                    ))
                    .style()
                    .width(Val::Percent(100.))
                    .height(Val::Percent(100.));
            },
        )
        .insert(MarkerWindow);

        self.enable_tooltip();
    }
}

#[derive(Component)]
struct Column(usize);

#[derive(Component)]
struct MarkerView;

fn setup_window(
    mut commands: Commands,
    mut events: EventWriter<SetColumnEvent>,
    query: Query<Entity, With<MarkerList>>,
) {
    commands.ui_builder(query.single()).insert(MarkerView);

    events.send(SetColumnEvent {
        column_id: 0,
        marker_id: None,
    });
}

fn remove_window(mut commands: Commands, query: Query<Entity, With<MarkerList>>) {
    commands.entity(query.single()).despawn_descendants();
}

fn set_column(
    mut commands: Commands,
    mut events: EventReader<SetColumnEvent>,
    markers: Res<MarkerTree>,
    columns: Query<(Entity, &Column)>,
    marker_view: Query<Entity, With<MarkerView>>,
    mut ui_events: EventWriter<UiEvent>,
) {
    for SetColumnEvent {
        column_id,
        marker_id,
    } in events.read()
    {
        let next_column_id = column_id + 1;

        let count = columns.iter().len();
        println!("count: {:?}", count);

        // Remove an existing columns with this column ID or higher.
        for (entity, _column) in columns
            .iter()
            .filter(|(_entity, column)| column.0 > *column_id)
        {
            commands.entity(entity).despawn_recursive();
        }

        let marker = marker_id
            .as_ref()
            .and_then(|marker_id| markers.get(marker_id));

        let label = marker
            .map(|marker| marker.label.clone())
            .unwrap_or("Top".to_string());

        let iter = marker
            .map(|marker| markers.iter(&marker.id).collect())
            .unwrap_or(markers.roots());

        commands
            .ui_builder(marker_view.single())
            .scroll_view(None, |scroll_view| {
                scroll_view.column(|parent| {
                    parent.label(LabelConfig::from(label));
                    for item in iter {
                        parent.marker_button(&item.id, &item.label, next_column_id);
                    }
                });
            })
            .insert(Column(next_column_id))
            .style()
            .width(Val::Px(200.));
    }
}

#[derive(Event, Debug)]
enum CheckboxEvent {
    Enable(String),
    Disable(String),
}

impl CheckboxEvent {
    fn id(&self) -> &str {
        match self {
            CheckboxEvent::Enable(id) => id,
            CheckboxEvent::Disable(id) => id,
        }
    }

    fn enabled(&self) -> bool {
        match self {
            CheckboxEvent::Enable(_) => true,
            CheckboxEvent::Disable(_) => false,
        }
    }
}

fn checkbox(
    query: Query<(&Checkbox, &MarkerItem), Changed<Checkbox>>,
    mut checkbox_events: EventWriter<CheckboxEvent>,
    mut ui_events: EventWriter<UiEvent>,
    markers: Res<MarkerTree>,
) {
    for (checkbox, item) in query.iter() {
        if checkbox.checked {
            ui_events.send(UiEvent::LoadMarker(item.id.clone()));
            checkbox_events.send_batch(
                markers
                    .iter(&item.id)
                    .map(|item| CheckboxEvent::Enable(item.id.to_string())),
            );
        } else {
            ui_events.send(UiEvent::UnloadMarker(item.id.clone()));
            checkbox_events.send_batch(
                markers
                    .iter(&item.id)
                    .map(|item| CheckboxEvent::Disable(item.id.to_string())),
            );
        }
    }
}

fn checkbox_events(
    mut query: Query<(&mut Checkbox, &MarkerItem)>,
    mut checkbox_events: EventReader<CheckboxEvent>,
) {
    for event in checkbox_events.read() {
        let event_id = event.id().to_string();

        if let Some(mut checkbox) = query.iter_mut().find_map(|(checkbox, item)| {
            if item.id == event_id {
                Some(checkbox)
            } else {
                None
            }
        }) {
            checkbox.checked = event.enabled();
        }
    }
}

fn toggle_show_ui(
    mut events: EventReader<UiEvent>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut ui: Query<&mut FloatingPanelConfig, With<MarkerWindow>>,
) {
    for event in events.read() {
        if let UiEvent::ToggleUI = event {
            let mut window = window.single_mut();
            let visible = !window.cursor.hit_test;
            if visible {
                window.cursor.hit_test = true;
                ui.single_mut().folded = false;
                info!("UI enabled");
            } else {
                window.cursor.hit_test = false;
                ui.single_mut().folded = true;
                info!("UI disabled");
            }
        }
    }
}

fn menu_interaction(
    query: Query<(&MenuItem, &OrrientMenuItem), Changed<MenuItem>>,
    mut events: EventWriter<UiEvent>,
) {
    for (menu_item, orrient_menu_item) in &query {
        if menu_item.interacted() {
            events.send(orrient_menu_item.0.clone());
        }
    }
}
