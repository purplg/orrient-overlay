use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bincode::Options as _;
use orrient_link::SocketMessage;

use std::{net::UdpSocket, ops::Deref};

use crate::ChannelRx;

#[derive(Resource)]
struct LinkSocket(UdpSocket);
impl Deref for LinkSocket {
    type Target = UdpSocket;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn setup(mut commands: Commands) {
    commands.insert_resource(LinkSocket(UdpSocket::bind("127.0.0.1:0").unwrap()));
}

/// Send MumbleLinkMessages over socket
fn socket_system(rx: Res<ChannelRx<SocketMessage>>, socket: Res<LinkSocket>) {
    while let Ok(message) = rx.try_recv() {
        match bincode::DefaultOptions::new()
            .allow_trailing_bytes()
            .serialize(&message)
        {
            Ok(buf) => {
                let _ = socket.send_to(buf.as_slice(), "127.0.0.1:5001");
            }
            Err(err) => {
                println!("err: {:?}", err);
            }
        }
    }
}

pub(crate) struct Plugin;

impl bevy_app::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, socket_system);
    }
}
