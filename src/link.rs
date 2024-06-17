use bevy::prelude::*;
use crossbeam_channel::Receiver;
use std::net::UdpSocket;

use mumblelink::{MumbleLinkDataDef, MumbleLinkMessage};

fn run(tx: crossbeam_channel::Sender<MumbleLinkMessage>) {
    let socket = UdpSocket::bind("127.0.0.1:5001").unwrap();
    loop {
        let mut buf = [0; 240];
        let _size = socket.recv(&mut buf);
        let message: MumbleLinkMessage = bincode::deserialize(&buf).unwrap();
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
        app.init_resource::<MumbleData>();
        app.add_event::<MumbleLinkEvent>();
        app.add_systems(Update, socket_system);
    }
}

#[derive(Resource, Deref)]
struct MumbleLinkMessageReceiver(pub Receiver<MumbleLinkMessage>);

#[derive(Event)]
pub enum MumbleLinkEvent {
    MumbleLinkData(MumbleLinkDataDef),
    Toggle,
    Save,
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct MumbleData(pub Option<MumbleLinkDataDef>);

fn socket_system(rx: Res<MumbleLinkMessageReceiver>, mut events: EventWriter<MumbleLinkEvent>) {
    let mut message: Option<MumbleLinkMessage> = None;

    // Only care about latest
    while let Ok(inner) = rx.try_recv() {
        message = Some(inner);
    }

    if let Some(message) = message {
        match message {
            MumbleLinkMessage::MumbleLinkData(data) => {
                events.send(MumbleLinkEvent::MumbleLinkData(data));
            }
            MumbleLinkMessage::Toggle => {
                events.send(MumbleLinkEvent::Toggle);
            }
            MumbleLinkMessage::Save => {
                events.send(MumbleLinkEvent::Save);
            }
        }
    }
}
