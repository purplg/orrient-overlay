use bevy::prelude::*;

use orrient_core::prelude::*;

use sickle_ui::prelude::*;
use sickle_ui::ui_builder::UiBuilder;

use crate::UiCamera;

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

#[derive(Component)]
struct AppStateText;

#[derive(Component)]
struct GameStateText;

trait UiDebugPanelExt {
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
                size: (270., 140.).into(),
                position: Some((2010., 0.).into()),
                ..default()
            },
            |parent| {
                // Player
                parent.row(|parent| {
                    parent.label(LabelConfig::from("Player"));
                });
                parent
                    .row(|parent| {
                        parent.column(|parent| {
                            parent.spawn((
                                TextBundle::from_section(
                                    "".to_string(),
                                    TextStyle {
                                        font_size: 14.,
                                        ..default()
                                    },
                                ),
                                DebugText::PlayerX,
                            ));
                        });
                        parent.column(|parent| {
                            parent.spawn((
                                TextBundle::from_section(
                                    "".to_string(),
                                    TextStyle {
                                        font_size: 14.,
                                        ..default()
                                    },
                                ),
                                DebugText::PlayerY,
                            ));
                        });
                        parent.column(|parent| {
                            parent.spawn((
                                TextBundle::from_section(
                                    "".to_string(),
                                    TextStyle {
                                        font_size: 14.,
                                        ..default()
                                    },
                                ),
                                DebugText::PlayerZ,
                            ));
                        });
                    })
                    .style()
                    .justify_content(JustifyContent::SpaceEvenly);

                // MapId
                parent.row(|parent| {
                    parent.label(LabelConfig::from("Map Id"));
                });
                parent.row(|parent| {
                    parent.column(|parent| {
                        parent.spawn((
                            TextBundle::from_section(
                                "".to_string(),
                                TextStyle {
                                    font_size: 14.,
                                    ..default()
                                },
                            ),
                            MapIdText,
                        ));
                    });
                });

                // AppState
                parent.row(|parent| {
                    parent.label(LabelConfig::from("AppState"));
                });
                parent.row(|parent| {
                    parent.column(|parent| {
                        parent.spawn((
                            TextBundle::from_section(
                                "".to_string(),
                                TextStyle {
                                    font_size: 14.,
                                    ..default()
                                },
                            ),
                            AppStateText,
                        ));
                    });
                });

                // GameState
                parent.row(|parent| {
                    parent.label(LabelConfig::from("GameState"));
                });
                parent.row(|parent| {
                    parent.column(|parent| {
                        parent.spawn((
                            TextBundle::from_section(
                                "".to_string(),
                                TextStyle {
                                    font_size: 14.,
                                    ..default()
                                },
                            ),
                            GameStateText,
                        ));
                    });
                });
            },
        )
        .insert(DebugPanel);
    }
}

fn update_player_position(
    mut query: Query<(&mut Text, &DebugText)>,
    mut events: EventReader<WorldEvent>,
) {
    for event in events.read() {
        if let WorldEvent::PlayerPositon(pos) = event {
            for (mut text, position_component) in &mut query {
                text.sections[0].value = match position_component {
                    DebugText::PlayerX => format!("x: {}", pos.x),
                    DebugText::PlayerY => format!("y: {}", pos.y),
                    DebugText::PlayerZ => format!("z: {}", pos.z),
                };
            }
        }
    }
}

fn update_map_id(mut query: Query<&mut Text, With<MapIdText>>, map_id: Res<MapId>) {
    let mut text = query.single_mut();
    text.sections[0].value = format!("{}", **map_id);
}

fn update_app_state(mut query: Query<&mut Text, With<AppStateText>>, state: Res<State<AppState>>) {
    let mut text = query.single_mut();
    text.sections[0].value = format!("{:?}", **state);
}

fn update_game_state(
    mut query: Query<&mut Text, With<GameStateText>>,
    state: Res<State<GameState>>,
) {
    let mut text = query.single_mut();
    text.sections[0].value = format!("{:?}", **state);
}

fn spawn_ui(mut commands: Commands, ui_camera: Res<UiCamera>) {
    commands.ui_builder(UiRoot).container(
        (NodeBundle::default(), TargetCamera(ui_camera.0)),
        |container| {
            container.debug_panel();
        },
    );
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ui);

        app.add_systems(
            Update,
            (
                update_player_position,
                update_map_id.run_if(resource_exists_and_changed::<MapId>),
                update_app_state.run_if(state_changed::<AppState>),
                update_game_state.run_if(state_changed::<GameState>),
            ),
        );
    }
}
