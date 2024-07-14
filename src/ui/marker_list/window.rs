use bevy::{prelude::*, window::PrimaryWindow};
use sickle_ui::{
    ui_builder::{UiBuilder, UiBuilderExt as _},
    ui_style::generated::*,
    widgets::prelude::*,
};

use crate::{marker::LoadedMarkers, parser::prelude::*, ui::OrrientMenuItem, UiEvent};

use super::{
    marker_button::UiMarkerButtonExt, separator::UiMarkerSeparatorExt, tooltip::UiToolTipExt as _,
};

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CheckboxEvent>();
        app.add_event::<MarkerWindowEvent>();
        app.add_systems(Update, menu_interaction);
        app.add_systems(Update, set_column);
        app.add_systems(Update, toggle_show_ui);
        app.add_systems(Update, checkbox);
        app.add_systems(Update, checkbox_events);
        app.add_systems(
            Update,
            (remove_window, setup_window)
                .chain()
                .run_if(resource_exists_and_changed::<MarkerPacks>),
        );
    }
}

#[derive(Event)]
pub(super) enum MarkerWindowEvent {
    SetRootColumn,
    SetColumn {
        column_id: usize,
        full_id: FullMarkerId,
    },
    ToggleMarkers,
}

#[derive(Component)]
pub(super) struct MarkerWindow;

#[derive(Component)]
struct MarkerList;

#[derive(Component)]
pub(super) struct MarkerItem {
    pub id: FullMarkerId,
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
                            parent
                                .menu_item(MenuItemConfig {
                                    name: "Unload all markers".into(),
                                    ..default()
                                })
                                .insert(OrrientMenuItem(UiEvent::UnloadAllMarkers));
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
    mut events: EventWriter<MarkerWindowEvent>,
    query: Query<Entity, With<MarkerList>>,
) {
    commands.ui_builder(query.single()).insert(MarkerView);

    events.send(MarkerWindowEvent::SetRootColumn);
}

fn remove_window(mut commands: Commands, query: Query<Entity, With<MarkerList>>) {
    commands.entity(query.single()).despawn_descendants();
}

fn set_column(
    mut commands: Commands,
    mut events: EventReader<MarkerWindowEvent>,
    packs: Res<MarkerPacks>,
    columns: Query<(Entity, &Column)>,
    marker_view: Query<Entity, With<MarkerView>>,
    loaded: Res<LoadedMarkers>,
) {
    let Ok(marker_view) = marker_view.get_single() else {
        return;
    };

    for event in events.read() {
        match event {
            MarkerWindowEvent::SetRootColumn => {
                commands
                    .ui_builder(marker_view)
                    .scroll_view(None, |scroll_view| {
                        scroll_view.column(|parent| {
                            parent.label(LabelConfig::from("Top"));
                            for (_pack_id, pack) in packs.iter() {
                                for marker in pack.roots().filter_map(|marker| pack.get(&marker.id))
                                {
                                    parent.marker_button(&pack, marker, 0, false);
                                }
                            }
                        });
                    })
                    .insert(Column(0))
                    .style()
                    .width(Val::Px(200.));
            }
            MarkerWindowEvent::SetColumn { column_id, full_id } => {
                let Some(pack) = packs.get(&full_id.pack_id) else {
                    warn!("Pack {} not found", full_id.pack_id);
                    continue;
                };

                let Some(parent_marker) = pack.get(&full_id.marker_id) else {
                    warn!("Marker {} not found", full_id.marker_id);
                    continue;
                };

                let markers = pack.iter(&full_id.marker_id).collect::<Vec<_>>();

                let next_column_id = column_id + 1;

                // Remove an existing columns with this column ID or higher.
                for (entity, _column) in columns
                    .iter()
                    .filter(|(_entity, column)| column.0 > *column_id)
                {
                    commands.entity(entity).despawn_recursive();
                }

                commands
                    .ui_builder(marker_view)
                    .scroll_view(None, |scroll_view| {
                        scroll_view.column(|parent| {
                            parent.label(LabelConfig::from(parent_marker.label.clone()));
                            for marker in &markers {
                                if let MarkerKind::Separator = marker.kind {
                                    parent.marker_separator(&marker.label);
                                } else {
                                    let full_id = full_id.with_marker_id(marker.id.clone());
                                    let checked = loaded.contains(&full_id);
                                    parent.marker_button(&pack, &marker, next_column_id, checked);
                                }
                            }
                        });
                    })
                    .insert(Column(next_column_id))
                    .style()
                    .width(Val::Px(200.));
            }
            MarkerWindowEvent::ToggleMarkers => {
                continue;
            }
        };
    }
}

#[derive(Event, Debug)]
enum CheckboxEvent {
    Enable(FullMarkerId),
    Disable(FullMarkerId),
}

impl CheckboxEvent {
    fn id(&self) -> &FullMarkerId {
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
    packs: Res<MarkerPacks>,
) {
    for (checkbox, item) in query.iter() {
        let Some(pack) = packs.get(&item.id.pack_id) else {
            continue;
        };

        if checkbox.checked {
            ui_events.send(UiEvent::LoadMarker(item.id.clone()));
            checkbox_events
                .send_batch(pack.iter(&item.id.marker_id).map(|marker| {
                    CheckboxEvent::Enable(item.id.with_marker_id(marker.id.clone()))
                }));
        } else {
            ui_events.send(UiEvent::UnloadMarker(item.id.clone().into()));
            checkbox_events.send_batch(
                pack.iter(&item.id.marker_id).map(|marker| {
                    CheckboxEvent::Disable(item.id.with_marker_id(marker.id.clone()))
                }),
            );
        }
    }
}

fn checkbox_events(
    mut query: Query<(&mut Checkbox, &MarkerItem)>,
    mut checkbox_events: EventReader<CheckboxEvent>,
) {
    for event in checkbox_events.read() {
        if let Some(mut checkbox) = query.iter_mut().find_map(|(checkbox, item)| {
            if &item.id == event.id() {
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
