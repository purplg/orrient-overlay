use bevy::prelude::*;
use sickle_ui::{prelude::*, ui_builder::UiBuilder};

use crate::{marker::MarkerTree, UiEvent};

use super::window::SetColumnEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentThemePlugin::<MarkerButton>::default());
        app.add_systems(Update, column_button);
    }
}

pub trait UiMarkerButtonExt {
    fn marker_button(
        &mut self,
        marker_id: impl Into<String>,
        label: impl Into<String>,
        column_id: usize,
    );
}

#[derive(Component)]
struct ColumnRef(usize);

#[derive(Component, Clone, Debug, Default, Reflect, UiContext)]
#[reflect(Component)]
pub struct MarkerButton(String);

impl DefaultTheme for MarkerButton {
    fn default_theme() -> Option<Theme<Self>> {
        Self::theme().into()
    }
}

impl MarkerButton {
    fn theme() -> Theme<MarkerButton> {
        let base_theme = PseudoTheme::deferred(None, MarkerButton::primary_style);
        Theme::new(vec![base_theme])
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
                hover: colors.container(Container::SurfaceHighest).into(),
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
        column_id: usize,
    ) {
        self.container(MarkerButton::frame(), |parent| {
            parent.spawn(TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 18.,
                    ..default()
                },
            ));
        })
        .insert((
            MarkerButton(marker_id.into()), //
            ColumnRef(column_id),
        ));
    }
}

#[derive(Component)]
struct Selected;

fn column_button(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &MarkerButton,
            &ColumnRef,
            &Interaction,
            Option<&Selected>,
        ),
        Changed<Interaction>,
    >,
    mut column_events: EventWriter<SetColumnEvent>,
    mut ui_events: EventWriter<UiEvent>,
    markers: Res<MarkerTree>,
) {
    for (entity, button, column_ref, interaction, selected) in &query {
        match interaction {
            Interaction::Pressed => {
                let marker_id = button.0.clone();
                if markers.iter(&button.0).count() > 0 {
                    column_events.send(SetColumnEvent {
                        column_id: column_ref.0,
                        marker_id: Some(marker_id),
                    });
                } else {
                    if selected.is_some() {
                        ui_events.send(UiEvent::UnloadMarker(marker_id));
                        commands.entity(entity).remove::<Selected>();
                    } else {
                        ui_events.send(UiEvent::LoadMarker(marker_id));
                        commands.entity(entity).insert(Selected);
                    }
                }
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}
