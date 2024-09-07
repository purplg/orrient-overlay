use bevy::color::palettes;
use bevy::prelude::*;

use orrient_core::prelude::*;
use orrient_pathing::prelude::*;

use sickle_ui::prelude::*;
use sickle_ui::ui_builder::UiBuilder;

use super::window::{MarkerItem, MarkerWindowEvent};

pub trait UiMarkerButtonExt {
    fn marker_button(
        &mut self,
        pack: &MarkerPack,
        marker: &Marker,
        column_id: usize,
        checked: bool,
    );
}

#[derive(Component)]
struct ColumnRef(usize);

#[derive(Component, Deref, Debug)]
struct MarkerCheckbox(FullMarkerId);

#[derive(Component, Clone, Default, Debug, UiContext)]
pub struct MarkerButton {
    full_id: FullMarkerId,
    has_children: bool,
    open: bool,
}

impl DefaultTheme for MarkerButton {
    fn default_theme() -> Option<Theme<Self>> {
        Self::theme().into()
    }
}

impl MarkerButton {
    fn theme() -> Theme<MarkerButton> {
        let base_theme = PseudoTheme::deferred(None, MarkerButton::primary_style);
        let open_theme =
            PseudoTheme::deferred_world(vec![PseudoState::Open], MarkerButton::open_style);
        let inactive_theme =
            PseudoTheme::deferred_world(vec![PseudoState::Closed], MarkerButton::inactive_style);
        let disabled_theme =
            PseudoTheme::deferred_world(vec![PseudoState::Disabled], MarkerButton::disabled_style);
        Theme::new(vec![base_theme, open_theme, inactive_theme, disabled_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();
        style_builder
            .border(UiRect::all(Val::Px(1.)))
            .border_color(Color::BLACK)
            .padding(UiRect::all(Val::Px(theme_spacing.gaps.small)))
            .animated()
            .background_color(AnimatedVals {
                idle: colors.container(Container::SurfaceMid),
                hover: colors.container(Container::SurfaceHigh).into(),
                ..default()
            });
    }

    fn open_style(style_builder: &mut StyleBuilder, _: Entity, _: &MarkerButton, world: &World) {
        let theme_data = world.resource::<ThemeData>().clone();
        let colors = theme_data.colors();
        style_builder.background_color(colors.container(Container::SurfaceHighest));
    }

    fn inactive_style(
        style_builder: &mut StyleBuilder,
        _: Entity,
        _: &MarkerButton,
        world: &World,
    ) {
        let theme_data = world.resource::<ThemeData>().clone();
        let colors = theme_data.colors();
        style_builder.animated().background_color(AnimatedVals {
            idle: colors.container(Container::SurfaceLowest),
            hover: colors.container(Container::SurfaceMid).into(),
            ..default()
        });
    }

    fn disabled_style(
        style_builder: &mut StyleBuilder,
        _: Entity,
        _: &MarkerButton,
        world: &World,
    ) {
        let theme_data = world.resource::<ThemeData>().clone();
        let colors = theme_data.colors();
        style_builder.background_color(colors.on(On::Error));
    }

    fn frame() -> impl Bundle {
        ButtonBundle::default()
    }
}

impl UiMarkerButtonExt for UiBuilder<'_, Entity> {
    fn marker_button(
        &mut self,
        pack: &MarkerPack,
        marker: &Marker,
        column_id: usize,
        checked: bool,
    ) {
        let full_id = pack.full_id(marker.id.clone());
        self.container(MarkerButton::frame(), |parent| {
            parent
                .row(|parent| {
                    parent
                        .checkbox(None, checked)
                        .insert(MarkerCheckbox(full_id.clone()));
                    parent.column(|parent| {
                        parent.spawn(TextBundle::from_section(
                            &marker.label,
                            TextStyle {
                                font_size: 14.,
                                ..default()
                            },
                        ));
                        parent.spawn(TextBundle::from_section(
                            &full_id.marker_id.to_string(),
                            TextStyle {
                                color: palettes::tailwind::GRAY_500.into(),
                                font_size: 10.,
                                ..default()
                            },
                        ));
                    });
                })
                .insert((
                    ColumnRef(column_id),
                    MarkerButton {
                        full_id,
                        has_children: pack.iter(&marker.id).count() > 0,
                        open: false,
                    },
                ));
        });
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

        if pack.contains_map_id(&button.full_id.marker_id, **map_id) {
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

                if pack.iter(&button.full_id.marker_id).count() == 0 {
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

fn checkbox_action(
    query: Query<(&Checkbox, &MarkerCheckbox, &FluxInteraction), Changed<FluxInteraction>>,
    mut events: EventWriter<MarkerEvent>,
    packs: Res<MarkerPacks>,
) {
    for (checkbox, full_id, interaction) in &query {
        if interaction.is_pressed() {
            if let Some(pack) = packs.get(&full_id.pack_id) {
                let markers = pack
                    .iter_recursive(&full_id.marker_id)
                    .map(|marker| full_id.with_marker_id(marker.id.clone()));

                if checkbox.checked {
                    events.send_batch(markers.map(MarkerEvent::Disable));
                } else {
                    events.send_batch(markers.map(MarkerEvent::Enabled));
                }
            }
        }
    }
}

fn checkbox_follow(
    mut query: Query<(&mut Checkbox, &MarkerCheckbox)>,
    mut events: EventReader<MarkerEvent>,
) {
    for event in events.read() {
        match event {
            MarkerEvent::Enabled(id_to_load) => {
                for (mut checkbox, this_id) in &mut query {
                    if checkbox.checked {
                        continue;
                    };
                    if this_id.within(id_to_load) {
                        checkbox.checked = true;
                    }
                }
            }
            MarkerEvent::Disable(id_to_unload) => {
                for (mut checkbox, this_id) in &mut query {
                    if !checkbox.checked {
                        continue;
                    };
                    if this_id.within(id_to_unload) {
                        checkbox.checked = false;
                    }
                }
            }
            MarkerEvent::DisableAll => {
                for (mut checkbox, _) in &mut query {
                    if !checkbox.checked {
                        continue;
                    };
                    checkbox.checked = false;
                }
            }
        }
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

        if !pack.contains_map_id(&button.full_id.marker_id, **map_id) {
            entity_cmds.add_pseudo_state(PseudoState::Disabled);
        }
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
    mut events: EventWriter<MarkerEvent>,
    packs: Res<MarkerPacks>,
) {
    for (checkbox, item) in query.iter() {
        let Some(pack) = packs.get(&item.id.pack_id) else {
            continue;
        };

        if checkbox.checked {
            events.send(MarkerEvent::Enabled(item.id.clone()));
            checkbox_events
                .send_batch(pack.iter(&item.id.marker_id).map(|marker| {
                    CheckboxEvent::Enable(item.id.with_marker_id(marker.id.clone()))
                }));
        } else {
            events.send(MarkerEvent::Disable(item.id.clone()));
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

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentThemePlugin::<MarkerButton>::default());
        app.add_systems(Update, button_interaction);
        app.add_systems(Update, checkbox_action);
        app.add_systems(Update, button_state);
        app.add_systems(Update, button_track_state);

        app.add_event::<CheckboxEvent>();
        app.add_systems(Update, checkbox);
        app.add_systems(Update, checkbox_events);

        app.add_systems(
            Update,
            button_mapid_disable.run_if(resource_exists_and_changed::<MapId>),
        );
        app.add_systems(Update, checkbox_follow.run_if(on_event::<MarkerEvent>()));
        app.add_systems(Update, button_init.run_if(resource_exists::<MapId>));
    }
}
