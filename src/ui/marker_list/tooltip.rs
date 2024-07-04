use bevy::{prelude::*, window::PrimaryWindow};
use manual::SetAbsolutePositionExt as _;
use sickle_ui::{ui_builder::UiBuilder, ui_style::*, widgets::prelude::*};

use super::window::MarkerItem;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (tool_tip_hover, tool_tip_position));
    }
}

#[derive(Component)]
struct ToolTipWindow;

pub trait UiToolTipExt {
    fn enable_tooltip(&mut self);
}

impl UiToolTipExt for UiBuilder<'_, Entity> {
    fn enable_tooltip(&mut self) {
        self.floating_panel(
            FloatingPanelConfig {
                title: None,
                draggable: false,
                resizable: false,
                foldable: false,
                closable: false,
                ..default()
            },
            FloatingPanelLayout {
                size: (300., 50.).into(),
                position: None,
                droppable: false,
            },
            |panel| {
                panel.spawn((
                    TextBundle::from_sections([
                        TextSection::new("", TextStyle::default()),
                        TextSection::new("", TextStyle::default()),
                    ]),
                    ToolTipText,
                ));
            },
        )
        .insert(ToolTipWindow);
    }
}

#[derive(Component)]
struct ToolTipText;

fn tool_tip_hover(
    mut tooltip: Query<&mut Visibility, With<ToolTipWindow>>,
    mut text: Query<&mut Text, With<ToolTipText>>,
    interaction: Query<(&MarkerItem, &Interaction)>,
) {
    let Ok(mut visibility) = tooltip.get_single_mut() else {
        return;
    };

    for (item, interaction) in &interaction {
        match interaction {
            Interaction::Hovered => {
                if *visibility == Visibility::Hidden {
                    *visibility = Visibility::Inherited;
                }
                if let Ok(mut text) = text.get_single_mut() {
                    if let Some(tip) = &item.tip {
                        if text.sections[0].value != *tip {
                            text.sections[0] = format!("{}\n", tip.as_str()).into();
                            text.sections[1] = item.description.clone().unwrap_or_default().into();
                        }
                    }
                }
                return;
            }
            _ => {}
        }
    }

    if *visibility != Visibility::Hidden {
        *visibility = Visibility::Hidden;
    }
}

fn tool_tip_position(
    mut commands: Commands,
    mut query: Query<Entity, With<ToolTipWindow>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    for entity in &mut query {
        if let Some(cursor) = window.single().cursor_position() {
            commands.style(entity).absolute_position(cursor);
        }
    }
}
