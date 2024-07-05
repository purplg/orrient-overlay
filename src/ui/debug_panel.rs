use bevy::prelude::*;
use sickle_ui::{ui_builder::UiBuilder, ui_style::generated::*, widgets::prelude::*};

use crate::WorldEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (update_player_position, update_map_id));
    }
}

#[derive(Component)]
struct DebugPanel;

#[derive(Component)]
enum DebugText {
    PlayerX,
    PlayerY,
    PlayerZ,
}

#[derive(Component)]
struct MapIdText;

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
                                DebugText::PlayerX,
                            ));
                        });
                        parent.column(|parent| {
                            parent.spawn((
                                TextBundle::from_section("".to_string(), TextStyle::default()),
                                DebugText::PlayerY,
                            ));
                        });
                        parent.column(|parent| {
                            parent.spawn((
                                TextBundle::from_section("".to_string(), TextStyle::default()),
                                DebugText::PlayerZ,
                            ));
                        });
                    })
                    .style()
                    .justify_content(JustifyContent::SpaceEvenly);

                parent.row(|parent| {
                    parent.label(LabelConfig::from("Map Id"));
                });
                parent.row(|parent| {
                    parent.column(|parent| {
                        parent.spawn((
                            TextBundle::from_section("".to_string(), TextStyle::default()),
                            MapIdText,
                        ));
                    });
                });
            },
        )
        .insert(DebugPanel)
        .style()
        .width(Val::Percent(100.));
    }
}

fn update_player_position(
    mut query: Query<(&mut Text, &DebugText)>,
    mut events: EventReader<WorldEvent>,
) {
    for event in events.read() {
        match event {
            WorldEvent::PlayerPositon(pos) => {
                for (mut text, position_component) in &mut query {
                    text.sections[0].value = match position_component {
                        DebugText::PlayerX => format!("x: {}", pos.x),
                        DebugText::PlayerY => format!("y: {}", pos.y),
                        DebugText::PlayerZ => format!("z: {}", pos.z),
                    };
                }
            }
            _ => {}
        }
    }
}

fn update_map_id(
    mut query: Query<&mut Text, With<MapIdText>>,
    mut events: EventReader<WorldEvent>,
) {
    for event in events.read() {
        match event {
            WorldEvent::MapUpdate(map_id) => {
                let mut text = query.single_mut();
                text.sections[0].value = format!("{}", map_id);
            }
            _ => {}
        }
    }
}
