use bevy::prelude::*;
use sickle_ui::{prelude::*, ui_builder::UiBuilder};

use crate::{marker::MarkerTree, UiEvent};

use super::window::MarkerWindowEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentThemePlugin::<MarkerButton>::default());
        app.add_systems(Update, column_button);
        app.add_systems(Update, checkbox_action);
        app.add_systems(Update, button_state);
    }
}

pub trait UiMarkerButtonExt {
    fn marker_button(
        &mut self,
        marker_id: impl Into<String>,
        label: impl Into<String>,
        has_children: bool,
        column_id: usize,
    );
}

#[derive(Component)]
struct ColumnRef(usize);

#[derive(Component, Clone, Debug, Default, Reflect, UiContext)]
#[reflect(Component)]
pub struct MarkerButton {
    marker_id: String,
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
        Theme::new(vec![base_theme, open_theme, inactive_theme])
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

    fn frame() -> impl Bundle {
        ButtonBundle {
            button: Button,
            ..default()
        }
    }
}

impl UiMarkerButtonExt for UiBuilder<'_, Entity> {
    fn marker_button(
        &mut self,
        marker_id: impl Into<String>,
        label: impl Into<String>,
        has_children: bool,
        column_id: usize,
    ) {
        let marker_id = marker_id.into();
        self.container(MarkerButton::frame(), |parent| {
            parent
                .row(|parent| {
                    parent.checkbox(None, false);
                    parent.spawn(TextBundle::from_section(
                        label,
                        TextStyle {
                            font_size: 14.,
                            ..default()
                        },
                    ));
                })
                .insert((
                    MarkerButton {
                        marker_id: marker_id.clone(),
                        has_children,
                        open: false,
                    }, //
                    ColumnRef(column_id), //
                ));
        });
    }
}

fn column_button(
    mut query: Query<(&mut MarkerButton, &ColumnRef, &Interaction), Changed<Interaction>>,
    mut column_events: EventWriter<MarkerWindowEvent>,
    markers: Res<MarkerTree>,
) {
    for (button, column_ref, interaction) in &mut query {
        match interaction {
            Interaction::Pressed => {
                if button.open {
                    continue;
                }

                if markers.iter(&button.marker_id).count() == 0 {
                    continue;
                }

                column_events.send(MarkerWindowEvent::SetColumn {
                    column_id: column_ref.0,
                    marker_id: Some(button.marker_id.clone()),
                });
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

fn button_state(
    mut commands: Commands,
    mut column_events: EventReader<MarkerWindowEvent>,
    mut buttons: Query<(Entity, &mut MarkerButton, &ColumnRef)>,
) {
    for event in column_events.read() {
        let MarkerWindowEvent::SetColumn {
            column_id,
            marker_id: Some(marker_id),
        } = event
        else {
            continue;
        };

        for (entity, mut button, column) in &mut buttons {
            // Only for a single column
            if *column_id != column.0 {
                continue;
            }

            if !button.has_children {
                continue;
            }

            button.open = *marker_id == button.marker_id;
            if button.open {
                commands
                    .entity(entity)
                    .remove_pseudo_state(PseudoState::Closed);
                commands.entity(entity).add_pseudo_state(PseudoState::Open);
            } else {
                commands
                    .entity(entity)
                    .remove_pseudo_state(PseudoState::Open);
                commands
                    .entity(entity)
                    .add_pseudo_state(PseudoState::Closed);
            }
        }
    }
}

fn checkbox_action(
    query: Query<(&Checkbox, &Parent), Changed<Checkbox>>,
    buttons: Query<&MarkerButton>,
    mut ui_events: EventWriter<UiEvent>,
) {
    for (checkbox, parent) in &query {
        let Ok(button) = buttons.get(**parent) else {
            return;
        };
        if checkbox.checked {
            ui_events.send(UiEvent::LoadMarker(button.marker_id.clone()));
        } else {
            ui_events.send(UiEvent::UnloadMarker(button.marker_id.clone()));
        }
    }
}
