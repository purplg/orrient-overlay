use crate::prelude::*;
use bevy::input::ButtonState;
use bincode::Options as _;
use crossbeam_channel::Receiver;
use orrient_input::{Action, ActionEvent};
use orrient_link::{MumbleLinkDataDef, SocketMessage};
use orrient_ui::UiEvent;
use std::net::UdpSocket;

fn run(tx: crossbeam_channel::Sender<SocketMessage>) {
    let socket = UdpSocket::bind("127.0.0.1:5001").unwrap();
    loop {
        let mut buf = [0; 240];
        let _size = socket.recv(&mut buf);
        let message = match bincode::DefaultOptions::new()
            .allow_trailing_bytes()
            .deserialize(&buf)
        {
            Ok(message) => message,
            Err(err) => {
                error!("Error decoding MumbleLink message: {:?}", err);
                continue;
            }
        };
        if let Err(e) = tx.send(message) {
            println!("e: {:?}", e);
        }
    }
}

#[derive(Resource, Deref)]
struct MumbleLinkMessageReceiver(pub Receiver<SocketMessage>);

fn start_socket_system(mut commands: Commands) {
    let (tx, rx) = crossbeam_channel::unbounded::<SocketMessage>();
    commands.insert_resource(MumbleLinkMessageReceiver(rx));
    std::thread::spawn(|| run(tx));
    debug!("Waiting for link...");
}

#[derive(Resource, Deref, DerefMut)]
struct PrevMumblelinkState(MumbleLinkDataDef);

fn monitor_system(
    mut commands: Commands,
    rx: Res<MumbleLinkMessageReceiver>,
    mut state: ResMut<NextState<AppState>>,
) {
    if let Ok(SocketMessage::MumbleLinkData(data)) = rx.try_recv() {
        if data.ui_tick > 0 {
            commands.insert_resource(MapId(data.identity.map_id));
            commands.insert_resource(PrevMumblelinkState(*data));
            state.set(AppState::Running);
            info!("Link connected.");
        }
    }
}

fn socket_system(
    mut commands: Commands,
    rx: Res<MumbleLinkMessageReceiver>,
    mut world_events: EventWriter<WorldEvent>,
    mut ui_events: EventWriter<UiEvent>,
    mut previous: ResMut<PrevMumblelinkState>,
) {
    while let Ok(message) = rx.try_recv() {
        match message {
            SocketMessage::MumbleLinkData(current) => {
                let facing = Vec3::new(
                    current.camera.front[0],
                    current.camera.front[1],
                    current.camera.front[2],
                );

                world_events.send(WorldEvent::CameraUpdate {
                    position: Vec3::new(
                        current.camera.position[0],
                        current.camera.position[1],
                        -current.camera.position[2],
                    ),
                    facing,
                    fov: current.identity.fov,
                });

                world_events.send(WorldEvent::PlayerPositon(Vec3 {
                    x: current.avatar.position[0],
                    y: current.avatar.position[1],
                    z: -current.avatar.position[2],
                }));

                if previous.context.compass_width != current.context.compass_width {
                    ui_events.send(UiEvent::CompassSize(UVec2 {
                        x: current.context.compass_width as u32,
                        y: current.context.compass_height as u32,
                    }));
                } else if previous.context.compass_height != current.context.compass_height {
                    ui_events.send(UiEvent::CompassSize(UVec2 {
                        x: current.context.compass_width as u32,
                        y: current.context.compass_height as u32,
                    }));
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

                if previous.context.map_open() != current.context.map_open() {
                    ui_events.send(UiEvent::MapOpen(current.context.map_open()));
                }

                if previous.context.map_id != current.identity.map_id {
                    commands.insert_resource(MapId(current.identity.map_id));
                }

                previous.0 = *current;
            }
            SocketMessage::Action(action) => match action {
                ActionEvent {
                    action,
                    state: ButtonState::Pressed,
                } => match action {
                    Action::Menu => {
                        ui_events.send(UiEvent::OpenUi);
                    }
                    Action::Close => {
                        ui_events.send(UiEvent::CloseUi);
                    }
                    _ => {}
                },
                _ => {}
            },
        }
    }
}

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::WaitingForMumbleLink), start_socket_system);
        app.add_systems(
            Update,
            monitor_system.run_if(in_state(AppState::WaitingForMumbleLink)),
        );
        app.add_systems(Update, socket_system.run_if(in_state(AppState::Running)));
    }
}
