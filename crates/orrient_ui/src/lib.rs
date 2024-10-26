pub mod compass;
// TODO
// mod console;
mod debug_panel;
mod guide_arrow;
mod input;
mod marker_list;

use bevy::{input::ButtonState, prelude::*};

use orrient_core::prelude::*;

use orrient_input::{Action, ActionEvent};
use orrient_link::{MumbleLinkDataDef, SocketMessage};
use sickle_ui::SickleUiPlugin;

#[derive(Event, Clone, Debug)]
pub enum UiEvent {
    ToggleUI,
    CloseUi,
    CompassSize(UVec2),
    PlayerPosition(Vec2),
    MapPosition(Vec2),
    MapScale(f32),
    MapOpen(bool),
}

#[derive(Resource)]
struct UiCamera(Entity);

fn setup_camera(mut commands: Commands) {
    let camera = commands
        .spawn(Camera3dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::srgba(0.0, 0.0, 0.0, 0.0)),
                ..default()
            },
            ..default()
        })
        .id();
    commands.insert_resource(UiCamera(camera));
}

fn ui_state_system(mut ui_events: EventReader<UiEvent>, mut state: ResMut<NextState<GameState>>) {
    for event in ui_events.read() {
        if let UiEvent::MapOpen(map_open) = event {
            if *map_open {
                state.set(GameState::WorldMap);
            } else {
                state.set(GameState::InGame);
            }
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
struct PrevMumblelinkState(MumbleLinkDataDef);

fn monitor_system(
    mut commands: Commands,
    mut socket_message: EventReader<SocketMessage>,
    mut state: ResMut<NextState<AppState>>,
) {
    for message in socket_message.read() {
        let mut data = if let SocketMessage::MumbleLinkData(data) = message {
            if data.ui_tick == 0 {
                continue;
            }
            data.clone()
        } else {
            continue;
        };

        commands.insert_resource(MapId(data.identity.map_id));
        data.context.compass_width = 0;
        data.context.compass_height = 0;
        commands.insert_resource(PrevMumblelinkState(*data.clone()));
        state.set(AppState::Running);
        info!("Link connected.");
        return;
    }
}

fn link_system(
    mut commands: Commands,
    mut socket_message: EventReader<SocketMessage>,
    mut ui_events: EventWriter<UiEvent>,
    mut previous: ResMut<PrevMumblelinkState>,
) {
    for message in socket_message.read() {
        match message {
            SocketMessage::MumbleLinkData(current) => {
                if previous.context.compass_width != current.context.compass_width
                    || previous.context.compass_height != current.context.compass_height
                {
                    ui_events.send(UiEvent::CompassSize(UVec2 {
                        x: current.context.compass_width as u32,
                        y: current.context.compass_height as u32,
                    }));
                }

                if previous.context.map_open() != current.context.map_open() {
                    ui_events.send(UiEvent::MapOpen(current.context.map_open()));
                }

                if previous.context.map_id != current.identity.map_id {
                    commands.insert_resource(MapId(current.identity.map_id));
                }

                ui_events.send(UiEvent::MapPosition(Vec2 {
                    x: current.context.map_center_x,
                    y: current.context.map_center_y,
                }));

                ui_events.send(UiEvent::PlayerPosition(Vec2 {
                    x: current.context.player_x,
                    y: current.context.player_y,
                }));

                ui_events.send(UiEvent::MapScale(current.context.map_scale));

                previous.0 = *current.clone();
            }
            SocketMessage::Action(action) => {
                if let ActionEvent {
                    action,
                    state: ButtonState::Pressed,
                } = action
                {
                    match action {
                        Action::Menu => {
                            ui_events.send(UiEvent::ToggleUI);
                        }
                        Action::Close => {
                            ui_events.send(UiEvent::CloseUi);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UiEvent>();

        // TODO
        // app.add_plugins(console::Plugin);
        app.add_plugins(SickleUiPlugin);
        app.add_plugins(compass::Plugin);
        app.add_plugins(guide_arrow::Plugin);
        app.add_plugins(marker_list::Plugin);
        app.add_plugins(debug_panel::Plugin);
        app.add_plugins(input::Plugin);

        app.add_systems(Update, link_system.run_if(in_state(AppState::Running)));
        app.add_systems(PreStartup, setup_camera);
        app.add_systems(
            Update,
            monitor_system.run_if(in_state(AppState::WaitingForMumbleLink)),
        );
        app.add_systems(
            Update,
            ui_state_system.run_if(in_state(AppState::Running).and_then(on_event::<UiEvent>())),
        );
    }
}
