use super::theme::*;
use crate::marker_list::window::MarkerWindowEvent;

use orrient_core::prelude::*;
use orrient_pathing::prelude::*;

use bevy::color::palettes;
use bevy::prelude::*;

use sickle_ui::prelude::*;
use sickle_ui::ui_builder::UiBuilder;

pub trait UiMarkerButtonExt {
    fn marker_button(
        &mut self,
        pack: &MarkerPack,
        marker_id: MarkerId,
        marker: &Marker,
        column_id: usize,
        checked: bool,
    );
}

#[derive(Component)]
struct ColumnRef(usize);

#[derive(Component, Debug)]
struct MarkerCheckbox(FullMarkerId);

impl UiMarkerButtonExt for UiBuilder<'_, Entity> {
    fn marker_button(
        &mut self,
        pack: &MarkerPack,
        marker_id: MarkerId,
        marker: &Marker,
        column_id: usize,
        checked: bool,
    ) {
        self.container(
            (
                MarkerButton::frame(),
                ColumnRef(column_id),
                MarkerButton {
                    full_id: pack.full_id(marker_id),
                    has_children: pack.iter(marker_id).count() > 0,
                    open: false,
                },
            ),
            |parent| {
                parent.row(|parent| {
                    parent
                        .checkbox(None, checked)
                        .insert(MarkerCheckbox(pack.full_id(marker_id)));
                    parent.column(|parent| {
                        parent.spawn(TextBundle::from_section(
                            &marker.label,
                            TextStyle {
                                font_size: 14.,
                                ..default()
                            },
                        ));
                        parent.spawn(TextBundle::from_section(
                            pack.name_of(marker_id).to_string(),
                            TextStyle {
                                color: palettes::tailwind::GRAY_500.into(),
                                font_size: 10.,
                                ..default()
                            },
                        ));
                    });
                });
            },
        );
    }
}

fn button_init(
    mut commands: Commands,
    buttons: Query<(Entity, &MarkerButton), Added<MarkerButton>>,
    packs: Res<MarkerPacks>,
    map_id: Res<MapId>,
) {
    for (entity, button) in &buttons {
        let mut entity_cmds = commands.entity(entity);
        let Some(pack) = packs.get(&button.full_id.pack_id) else {
            warn!("Pack for button not found.");
            return;
        };

        if !pack.contains_map_id(button.full_id.marker_id, **map_id) {
            entity_cmds.add_pseudo_state(PseudoState::Disabled);
        }
    }
}

fn button_mapid_disable(
    mut commands: Commands,
    buttons: Query<(Entity, &MarkerButton)>,
    map_id: Res<MapId>,
    packs: Res<MarkerPacks>,
) {
    for (entity, button) in &buttons {
        let mut entity_cmds = commands.entity(entity);
        let Some(pack) = packs.get(&button.full_id.pack_id) else {
            warn!("Button doesn't exist in pack.");
            continue;
        };

        if pack.contains_map_id(button.full_id.marker_id, **map_id) {
            entity_cmds.remove_pseudo_state(PseudoState::Disabled);
        } else {
            entity_cmds.add_pseudo_state(PseudoState::Disabled);
        }
    }
}

fn button_track_state(
    mut commands: Commands,
    buttons: Query<(Entity, &MarkerButton), Changed<MarkerButton>>,
) {
    for (entity, button) in &buttons {
        let mut entity_cmds = commands.entity(entity);

        if button.open {
            entity_cmds.remove_pseudo_state(PseudoState::Closed);
            entity_cmds.add_pseudo_state(PseudoState::Open);
        } else {
            entity_cmds.remove_pseudo_state(PseudoState::Open);
            entity_cmds.add_pseudo_state(PseudoState::Closed);
        }
    }
}

fn button_interaction(
    mut query: Query<(&mut MarkerButton, &ColumnRef, &Interaction), Changed<Interaction>>,
    mut column_events: EventWriter<MarkerWindowEvent>,
    packs: Res<MarkerPacks>,
) {
    for (button, column_ref, interaction) in &mut query {
        match interaction {
            Interaction::Pressed => {
                if button.open {
                    continue;
                }

                let Some(pack) = packs.get(&button.full_id.pack_id) else {
                    continue;
                };

                if pack.iter(button.full_id.marker_id).count() == 0 {
                    continue;
                }

                column_events.send(MarkerWindowEvent::SetColumn {
                    column_id: column_ref.0 + 1,
                    full_id: button.full_id.clone(),
                });
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

fn button_state(
    mut column_events: EventReader<MarkerWindowEvent>,
    mut buttons: Query<(&mut MarkerButton, &ColumnRef)>,
) {
    for event in column_events.read() {
        let MarkerWindowEvent::SetColumn { column_id, full_id } = event else {
            continue;
        };

        for (mut button, column) in &mut buttons {
            // Only for a single column
            if *column_id - 1 != column.0 {
                continue;
            }

            if !button.has_children {
                continue;
            }

            button.open = *full_id == button.full_id;
        }
    }
}

/// What happens when a checkbox is toggled
fn checkbox_action(
    query: Query<(&Checkbox, &MarkerCheckbox, &FluxInteraction), Changed<FluxInteraction>>,
    mut events: EventWriter<MarkerEvent>,
    packs: Res<MarkerPacks>,
) {
    for (checkbox, marker_checkbox, interaction) in &query {
        if !interaction.is_released() {
            continue;
        }

        let Some(pack) = packs.get(&marker_checkbox.0.pack_id) else {
            continue;
        };

        let markers = pack
            .recurse(marker_checkbox.0.marker_id)
            .map(|(id, _marker)| pack.full_id(id));
        for marker in markers {
            println!("marker: {:?}", marker.marker_name);
        }

        let markers = pack
            .recurse(marker_checkbox.0.marker_id)
            .map(|(id, _marker)| pack.full_id(id));

        if checkbox.checked {
            events.send_batch(markers.map(MarkerEvent::Disable));
        } else {
            events.send_batch(markers.map(MarkerEvent::Enable));
        }
    }
}

/// Update the state of checkboxes as markers get enabled/disabled
fn checkbox_update(
    mut query: Query<(&mut Checkbox, &MarkerCheckbox)>,
    mut r_marker_events: EventReader<MarkerEvent>,
) {
    for event in r_marker_events.read() {
        match event {
            MarkerEvent::Enable(full_marker_id) => {
                if let Some((mut checkbox, _marker_id)) = query
                    .iter_mut()
                    .find(|(_checkbox, marker_id)| &marker_id.0 == full_marker_id)
                {
                    checkbox.checked = true;
                }
            }
            MarkerEvent::Disable(full_marker_id) => {
                if let Some((mut checkbox, _marker_id)) = query
                    .iter_mut()
                    .find(|(_checkbox, marker_id)| &marker_id.0 == full_marker_id)
                {
                    checkbox.checked = false;
                }
            }
            MarkerEvent::DisableAll => {
                for (mut checkbox, _marker_id) in &mut query {
                    checkbox.checked = false;
                }
            }
        }
    }
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentThemePlugin::<MarkerButton>::default());
        app.add_systems(Update, button_interaction);
        app.add_systems(Update, button_state);
        app.add_systems(Update, button_track_state);

        app.add_systems(Update, checkbox_update.run_if(on_event::<MarkerEvent>()));
        app.add_systems(Update, checkbox_action.after(checkbox_update));

        app.add_systems(
            Update,
            button_mapid_disable.run_if(resource_exists_and_changed::<MapId>),
        );
        app.add_systems(Update, button_init.run_if(resource_exists::<MapId>));
    }
}
