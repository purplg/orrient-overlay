use std::f32::consts::PI;
use std::net::UdpSocket;

use bevy::app::AppExit;
use bevy::color::palettes::basic::{BLUE, RED};
use bevy::input::keyboard::KeyboardInput;
use bevy::window::{CompositeAlphaMode, PrimaryWindow, WindowResolution};
use bevy::{
    prelude::*,
    window::{Cursor, WindowLevel},
};
use crossbeam_channel::Receiver;
use mumblelink::{MumbleLinkDataDef, MumbleLinkMessage};

fn main() {
    let (tx, rx) = crossbeam_channel::unbounded::<MumbleLinkMessage>();

    // launch_gw();
    std::thread::spawn(|| link(tx));
    launch_bevy(rx);
}

fn link(tx: crossbeam_channel::Sender<MumbleLinkMessage>) {
    let socket = UdpSocket::bind("127.0.0.1:5001").unwrap();
    loop {
        let mut buf = [0; 4096];
        let _ = socket.recv(&mut buf);
        let message: MumbleLinkMessage = bincode::deserialize(&buf).unwrap();
        if let Err(e) = tx.send(message) {
            println!("e: {:?}", e);
        }
    }
}

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
        }
    }
}

#[derive(Resource, Deref)]
struct MumbleLinkMessageReceiver(Receiver<MumbleLinkMessage>);

#[derive(Resource, Default)]
struct MumbleData(Option<MumbleLinkDataDef>);

#[derive(Event)]
enum MumbleLinkEvent {
    MumbleLinkData(MumbleLinkDataDef),
    Toggle,
}

fn launch_bevy(rx: crossbeam_channel::Receiver<MumbleLinkMessage>) {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "GW2Orrient".to_string(),
            resolution: WindowResolution::new(2560., 1440.),
            transparent: true,
            // decorations: false,
            window_level: WindowLevel::AlwaysOnTop,
            composite_alpha_mode: CompositeAlphaMode::PreMultiplied,
            cursor: Cursor {
                hit_test: true,
                ..default()
            },
            ..default()
        }),
        ..default()
    }));

    app.insert_resource(ClearColor(Color::NONE));
    app.insert_resource(MumbleLinkMessageReceiver(rx));
    app.init_resource::<MumbleData>();
    app.add_event::<MumbleLinkEvent>();

    app.add_systems(Startup, setup);
    app.add_systems(Update, gizmo);
    app.add_systems(Update, socket_system);
    app.add_systems(Update, camera_system);
    app.add_systems(
        Update,
        toggle_hittest_system.run_if(on_event::<MumbleLinkEvent>()),
    );
    app.add_systems(Update, input.run_if(on_event::<KeyboardInput>()));

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera3dBundle::default());
}

fn camera_system(
    mut events: EventReader<MumbleLinkEvent>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    for event in events.read() {
        if let MumbleLinkEvent::MumbleLinkData(mumbledata) = event {
            let mut pos = camera.single_mut();
            let position = Vec3::new(
                mumbledata.camera.position[0],
                mumbledata.camera.position[1],
                mumbledata.camera.position[2],
            );
            pos.translation = position;

            pos.look_to(
                -Dir3::new_unchecked(Vec3::new(
                    mumbledata.camera.front[0],
                    mumbledata.camera.front[1],
                    mumbledata.camera.front[2],
                )),
                Vec3::Y,
            );
        }
    }
}

fn toggle_hittest_system(
    mut events: EventReader<MumbleLinkEvent>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
    for event in events.read() {
        if let MumbleLinkEvent::Toggle = event {
            window.single_mut().cursor.hit_test = !window.single_mut().cursor.hit_test;
            println!("hittest: {:?}", window.single_mut().cursor.hit_test);
        }
    }
}

fn gizmo(
    mut gizmos: Gizmos,
    mut events: EventReader<MumbleLinkEvent>,
    mut mumbledata: ResMut<MumbleData>,
) {
    let position = Vec3::new(-100.0, 25.0, 315.0);
    gizmos.sphere(position, Quat::default(), 10.0, RED);

    for event in events.read() {
        if let MumbleLinkEvent::MumbleLinkData(data) = event {
            mumbledata.0 = Some(data.clone());
        }
    }

    if let Some(data) = &mumbledata.0 {
        let player = Vec3::new(
            data.avatar.position[0],
            data.avatar.position[1],
            data.avatar.position[2],
        );
        gizmos.sphere(player, Quat::default(), 1.0, BLUE);
    }
}

fn input(
    input: Res<ButtonInput<KeyCode>>,
    mut mumble_link_event: EventWriter<MumbleLinkEvent>,
) {
    if input.just_pressed(KeyCode::Escape) {
        mumble_link_event.send(MumbleLinkEvent::Toggle);
    }
}
