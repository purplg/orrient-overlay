use bevy::prelude::*;
use sickle_ui::{ui_builder::UiBuilder, ui_style::generated::*, widgets::prelude::*};

use crate::player::Player;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update);
    }
}

#[derive(Component)]
struct DebugPanel;

#[derive(Component)]
enum PlayerPosition {
    X,
    Y,
    Z,
}

pub trait UiDebugPanelExt {
    fn debug_panel(&mut self);
}

impl UiDebugPanelExt for UiBuilder<'_, Entity> {
    fn debug_panel(&mut self) {
        self.floating_panel(
            FloatingPanelConfig {
                title: Some("Debug".into()),
                ..default()
            },
            FloatingPanelLayout {
                size: (300., 100.).into(),
                position: Some((2000., 100.).into()),
                ..default()
            },
            |parent| {
                parent.row(|parent| {
                    parent.label(LabelConfig::from("Player"));
                });
                parent
                    .row(|parent| {
                        parent.column(|parent| {
                            parent.spawn((
                                TextBundle::from_section("".to_string(), TextStyle::default()),
                                PlayerPosition::X,
                            ));
                        });
                        parent.column(|parent| {
                            parent.spawn((
                                TextBundle::from_section("".to_string(), TextStyle::default()),
                                PlayerPosition::Y,
                            ));
                        });
                        parent.column(|parent| {
                            parent.spawn((
                                TextBundle::from_section("".to_string(), TextStyle::default()),
                                PlayerPosition::Z,
                            ));
                        });
                    })
                    .style()
                    .justify_content(JustifyContent::SpaceEvenly);
            },
        )
        .insert(DebugPanel)
        .style()
        .width(Val::Percent(100.));
    }
}

fn update(mut query: Query<(&mut Text, &PlayerPosition)>, player: Query<&Transform, With<Player>>) {
    let pos = player.single();
    for (mut text, position_component) in &mut query {
        text.sections[0].value = match position_component {
            PlayerPosition::X => format!("x: {}", pos.translation.x),
            PlayerPosition::Y => format!("y: {}", pos.translation.y),
            PlayerPosition::Z => format!("z: {}", pos.translation.z),
        };
    }
}
