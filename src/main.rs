use std::f32::consts::PI;
use std::net::UdpSocket;

use bevy::app::AppExit;
use bevy::color::palettes::basic::{BLUE, RED};
use bevy::input::keyboard::KeyboardInput;
use bevy::window::{CompositeAlphaMode, WindowResolution};
use bevy::{
    prelude::*,
    window::{Cursor, WindowLevel},
};
use crossbeam_channel::Receiver;
use mumblelink::MumbleLinkDataDef;

fn main() {
    let (tx, rx) = crossbeam_channel::unbounded::<MumbleLinkDataDef>();

    // launch_gw();
    std::thread::spawn(|| link(tx));
    launch_bevy(rx);
}

fn link(tx: crossbeam_channel::Sender<MumbleLinkDataDef>) {
    let socket = UdpSocket::bind("127.0.0.1:5001").unwrap();
    loop {
        let mut buf = [0; 4096];
        let _ = socket.recv(&mut buf);
        let data: MumbleLinkDataDef = bincode::deserialize(&buf).unwrap();
        if let Err(e) = tx.send(data) {
            println!("e: {:?}", e);
        }
    }
}

fn socket_system(rx: Res<MumbleDataReceiver>, mut mumbledata: ResMut<MumbleData>) {
    let mut data: Option<MumbleLinkDataDef> = None;

    // Only care about latest
    while let Ok(inner) = rx.try_recv() {
        data = Some(inner);
    }

    if let Some(data) = data {
        mumbledata.0 = Some(data.into());
    }
}

#[derive(Resource, Deref)]
struct MumbleDataReceiver(Receiver<MumbleLinkDataDef>);

#[derive(Resource, Default)]
struct MumbleData(Option<MumbleLinkDataDef>);

fn launch_bevy(rx: crossbeam_channel::Receiver<MumbleLinkDataDef>) {
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
                hit_test: false,
                ..default()
            },
            ..default()
        }),
        ..default()
    }));

    app.insert_resource(ClearColor(Color::NONE));
    app.insert_resource(MumbleDataReceiver(rx));
    app.init_resource::<MumbleData>();

    app.add_systems(Startup, setup);
    app.add_systems(Update, gizmo);
    app.add_systems(Update, socket_system);
    app.add_systems(Update, camera_system);
    app.add_systems(Update, input.run_if(on_event::<KeyboardInput>()));

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera3dBundle::default());
}

fn camera_system(mumbledata: Res<MumbleData>, mut camera: Query<&mut Transform, With<Camera3d>>) {
    let Some(mumbledata) = &mumbledata.0 else {
        return;
    };

    let mut pos = camera.single_mut();
    let position = Vec3::new(
        mumbledata.camera.position[0],
        mumbledata.camera.position[1],
        mumbledata.camera.position[2],
    );
    pos.translation = position;

    pos.look_to(
        Dir3::new_unchecked(Vec3::new(
            mumbledata.camera.front[0],
            mumbledata.camera.front[1],
            mumbledata.camera.front[2],
        )),
        Vec3::Y,
    );
}

fn gizmo(mut gizmos: Gizmos, mumbledata: Res<MumbleData>) {
    let position = Vec3::new(-100.0, 25.0, 315.0);
    gizmos.sphere(position, Quat::default(), 10.0, RED);

    if let Some(mumbledata) = &mumbledata.0 {
        let player = Vec3::new(
            mumbledata.avatar.position[0],
            mumbledata.avatar.position[1],
            mumbledata.avatar.position[2],
        );
        gizmos.sphere(player, Quat::default(), 1.0, BLUE);
    };
}

fn input(mut events: EventReader<KeyboardInput>, mut app_exit_events: ResMut<Events<AppExit>>) {
    for event in events.read() {
        match event.key_code {
            KeyCode::Escape => {
                app_exit_events.send(AppExit::Success);
            }
            _ => {}
        }
    }
}
