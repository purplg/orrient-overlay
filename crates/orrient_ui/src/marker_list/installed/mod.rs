mod theme;

pub(super) mod marker_button;
pub(super) mod separator;
pub(super) mod tooltip;

use marker_button::UiMarkerButtonExt as _;
use orrient_pathing::prelude::EnabledMarkers;
use orrient_pathing::prelude::MarkerEvent;
use orrient_pathing::prelude::MarkerKind;
use orrient_pathing::prelude::MarkerPacks;
use orrient_pathing::prelude::ReloadMarkersEvent;
use separator::UiMarkerSeparatorExt as _;

use super::window::MarkerWindowEvent;
use super::window::OrrientMenuItem;

use bevy::prelude::*;
use sickle_ui::prelude::*;

#[derive(Component)]
struct Column(usize);

fn spawn_view(mut events: EventWriter<MarkerWindowEvent>) {
    events.send(MarkerWindowEvent::SetRootColumn);
}

fn clear_view(mut commands: Commands, query: Query<Entity, With<InstalledView>>) {
    if let Ok(entity) = query.get_single() {
        commands.entity(entity).despawn_descendants();
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

fn refresh(_: Trigger<ReloadMarkersEvent>, mut ew_window_event: EventWriter<MarkerWindowEvent>) {
    ew_window_event.send(MarkerWindowEvent::SetRootColumn);
}

fn set_column(
    mut commands: Commands,
    mut events: EventReader<MarkerWindowEvent>,
    packs: Res<MarkerPacks>,
    columns: Query<(Entity, &Column)>,
    marker_view: Query<Entity, With<InstalledView>>,
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
                    .scroll_view(Some(ScrollAxis::Vertical), |scroll_view| {
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
                    .scroll_view(Some(ScrollAxis::Vertical), |parent| {
                        parent.column(|parent| {
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
        };
    }
}

pub(super) use theme::InstalledView;
pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(theme::Plugin);
        app.add_plugins(tooltip::Plugin);
        app.add_plugins(marker_button::Plugin);
        app.add_plugins(separator::Plugin);

        app.add_systems(
            Update,
            (clear_view, spawn_view)
                .chain()
                .run_if(resource_exists_and_changed::<MarkerPacks>),
        );
        app.add_systems(Update, menu_interaction);
        app.add_systems(Update, set_column);
        app.observe(refresh);
    }
}