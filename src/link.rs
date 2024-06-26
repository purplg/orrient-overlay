use bevy::prelude::*;
use crossbeam_channel::Receiver;
use std::net::UdpSocket;

use mumblelink::MumbleLinkMessage;

use crate::OrrientEvent;

fn run(tx: crossbeam_channel::Sender<MumbleLinkMessage>) {
    let socket = UdpSocket::bind("127.0.0.1:5001").unwrap();
    loop {
        let mut buf = [0; 240];
        let _size = socket.recv(&mut buf);
        let message = match bincode::deserialize(&buf) {
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

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = crossbeam_channel::unbounded::<MumbleLinkMessage>();

        std::thread::spawn(|| run(tx));

        app.insert_resource(MumbleLinkMessageReceiver(rx));
        app.add_systems(Update, socket_system);
    }
}

#[derive(Resource, Deref)]
struct MumbleLinkMessageReceiver(pub Receiver<MumbleLinkMessage>);

fn socket_system(rx: Res<MumbleLinkMessageReceiver>, mut events: EventWriter<OrrientEvent>) {
    let mut message: Option<MumbleLinkMessage> = None;

    // Only care about latest
    while let Ok(inner) = rx.try_recv() {
        message = Some(inner);
    }

    let Some(message) = message else {
        return;
    };

    match message {
        MumbleLinkMessage::MumbleLinkData(data) => {
            // events.send(MumbleLinkEvent::Data(data));
            let facing = Vec3::new(
                data.camera.front[0],
                data.camera.front[1],
                data.camera.front[2],
            );

            events.send(OrrientEvent::CameraUpdate {
                position: Vec3::new(
                    data.camera.position[0],
                    data.camera.position[1],
                    -data.camera.position[2],
                ),
                facing,
                fov: data.identity.fov,
            });

            events.send(OrrientEvent::PlayerPositon(Vec3 {
                x: data.avatar.position[0],
                y: data.avatar.position[1],
                z: -data.avatar.position[2],
            }));
        }
        MumbleLinkMessage::Toggle => {
            events.send(OrrientEvent::ToggleUI);
        }
        MumbleLinkMessage::Save => {
            events.send(OrrientEvent::SavePosition);
        }
    }
}
