use bevy::prelude::*;

use orrient_pathing::marker::MapMarkers;
use orrient_pathing::marker::EnabledMarkers;
use orrient_pathing::prelude::*;

use super::marker_button::UiMarkerButtonExt as _;
use super::separator::UiMarkerSeparatorExt as _;
use super::tooltip::UiToolTipExt as _;
use crate::OrrientMenuItem;
use crate::UiEvent;
use bevy::window::PrimaryWindow;
use sickle_ui::ui_builder::UiBuilder;
use sickle_ui::ui_builder::UiBuilderExt as _;
use sickle_ui::ui_style::generated::SetHeightExt as _;
use sickle_ui::ui_style::generated::SetWidthExt as _;
use sickle_ui::widgets::prelude::*;

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
                size: (1000., 1080.).into(),
                position: Some((100., 100.).into()),
                ..default()
            },
            |parent| {
                parent.menu_bar(|parent| {
                    parent.menu(
                        MenuConfig {
                            name: "Menu".into(),
                            ..default()
                        },
                        |parent| {
                            parent
                                .menu_item(MenuItemConfig {
                                    name: "Hide all markers".into(),
                                    ..default()
                                })
                                .insert(OrrientMenuItem(MarkerEvent::DisableAll));
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
    visible_markers: Res<EnabledMarkers>,
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
                                    parent.marker_button(pack, marker, 0, false);
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
                                    let checked = visible_markers.contains(&full_id);
                                    parent.marker_button(pack, marker, next_column_id, checked);
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

fn toggle_show_ui(
    mut events: EventReader<UiEvent>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut ui: Query<&mut FloatingPanelConfig, With<MarkerWindow>>,
) {
    for event in events.read() {
        match event {
            UiEvent::ToggleUI => {
                let mut window = window.single_mut();
                if window.cursor.hit_test {
                    window.cursor.hit_test = false;
                    ui.single_mut().folded = true;
                    info!("UI disabled");
                } else {
                    window.cursor.hit_test = true;
                    ui.single_mut().folded = false;
                    info!("UI enabled");
                }
            }
            UiEvent::CloseUi => {
                let mut window = window.single_mut();
                window.cursor.hit_test = false;
                ui.single_mut().folded = true;
                info!("UI disabled");
            }
            _ => {}
        }
    }
}

fn menu_interaction(
    query: Query<(&MenuItem, &OrrientMenuItem), Changed<MenuItem>>,
    mut events: EventWriter<MarkerEvent>,
) {
    for (menu_item, orrient_menu_item) in &query {
        if menu_item.interacted() {
            events.send(orrient_menu_item.0.clone());
        }
    }
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MarkerWindowEvent>();
        app.add_systems(Update, menu_interaction);
        app.add_systems(Update, set_column);
        app.add_systems(Update, toggle_show_ui);
        app.add_systems(
            Update,
            (remove_window, setup_window)
                .chain()
                .run_if(resource_exists_and_changed::<MarkerPacks>),
        );
    }
}
