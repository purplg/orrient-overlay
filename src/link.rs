use bevy::prelude::*;
use bincode::Options as _;
use crossbeam_channel::Receiver;
use std::{
    net::UdpSocket,
    ops::{Deref, DerefMut},
};

use orrient_link::MumbleLinkMessage;

use crate::{UiEvent, WorldEvent};

fn run(tx: crossbeam_channel::Sender<MumbleLinkMessage>) {
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

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        let (tx, rx) = crossbeam_channel::unbounded::<MumbleLinkMessage>();

        std::thread::spawn(|| run(tx));

        app.insert_resource(MumbleLinkMessageReceiver(rx));
        app.add_systems(Update, socket_system);
    }
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct MapId(pub u32);

impl Deref for MapId {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Resource, Deref)]
struct MumbleLinkMessageReceiver(pub Receiver<MumbleLinkMessage>);

fn socket_system(
    mut commands: Commands,
    rx: Res<MumbleLinkMessageReceiver>,
    mut world_events: EventWriter<WorldEvent>,
    mut ui_events: EventWriter<UiEvent>,
    mut prev_mapid: Local<u32>,
) {
    while let Ok(message) = rx.try_recv() {
        match message {
            MumbleLinkMessage::MumbleLinkData(data) => {
                let facing = Vec3::new(
                    data.camera.front[0],
                    data.camera.front[1],
                    data.camera.front[2],
                );

                world_events.send(WorldEvent::CameraUpdate {
                    position: Vec3::new(
                        data.camera.position[0],
                        data.camera.position[1],
                        -data.camera.position[2],
                    ),
                    facing,
                    fov: data.identity.fov,
                });

                world_events.send(WorldEvent::PlayerPositon(Vec3 {
                    x: data.avatar.position[0],
                    y: data.avatar.position[1],
                    z: -data.avatar.position[2],
                }));

                if *prev_mapid != data.identity.map_id {
                    commands.insert_resource(MapId(data.identity.map_id));
                    *prev_mapid = data.identity.map_id;
                }
            }
            MumbleLinkMessage::Toggle => {
                ui_events.send(UiEvent::ToggleUI);
            }
            MumbleLinkMessage::Save => {
                world_events.send(WorldEvent::SavePosition);
            }
        }
    }
}
