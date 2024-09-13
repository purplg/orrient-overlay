mod structs;
pub use structs::*;

use orrient_core::prelude::*;

use bevy::prelude::*;

use bincode::Options as _;
use crossbeam_channel::Receiver;
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
            println!("Error when sending to Mumblelink: {:?}", e);
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

fn socket_system(
    rx: Res<MumbleLinkMessageReceiver>,
    mut events: EventWriter<SocketMessage>,
    mut world_events: EventWriter<WorldEvent>,
) {
    while let Ok(message) = rx.try_recv() {
        events.send(message.clone());
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
            }
            SocketMessage::Action(_) => {}
        }
    }
}

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SocketMessage>();
        app.add_systems(OnEnter(AppState::WaitingForMumbleLink), start_socket_system);
        app.add_systems(Update, socket_system);
    }
}
