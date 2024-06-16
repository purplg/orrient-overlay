use std::net::UdpSocket;

use bevy::color::palettes::basic;
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
            MumbleLinkMessage::Save => {
                events.send(MumbleLinkEvent::Save);
            }
        }
    }
}

#[derive(Resource, Deref)]
struct MumbleLinkMessageReceiver(Receiver<MumbleLinkMessage>);

#[derive(Resource, Default)]
struct MumbleData(Option<MumbleLinkDataDef>);

#[derive(Component)]
struct SavedPosition;

#[derive(Event)]
enum MumbleLinkEvent {
    MumbleLinkData(MumbleLinkDataDef),
    Toggle,
    Save,
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
                hit_test: false,
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
    app.add_systems(Update, save_pos_system);
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
            pos.translation = Vec3::new(
                mumbledata.camera.position[0],
                mumbledata.camera.position[1],
                mumbledata.camera.position[2],
            );

            pos.look_to(
                Dir3::new(Vec3::new(
                    mumbledata.camera.front[0],
                    mumbledata.camera.front[1],
                    mumbledata.camera.front[2],
                ))
                .unwrap(),
                Vec3::new(
                    mumbledata.camera.top[0],
                    mumbledata.camera.top[1],
                    mumbledata.camera.top[2],
                ),
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
            let mut window = window.single_mut();
            window.cursor.hit_test = !window.cursor.hit_test;
            window.decorations = window.cursor.hit_test;
            println!("hittest: {:?}", window.cursor.hit_test);
        }
    }
}

fn save_pos_system(
    mut commands: Commands,
    mut events: EventReader<MumbleLinkEvent>,
    mumbledata: Res<MumbleData>,
    mut query: Query<&mut Transform, With<SavedPosition>>,
) {
    for event in events.read() {
        if let MumbleLinkEvent::Save = event {
            if let Some(data) = &mumbledata.0 {
                if let Ok(mut pos) = query.get_single_mut() {
                    pos.translation = Vec3::new(
                        data.avatar.position[0],
                        data.avatar.position[1],
                        data.avatar.position[2],
                    );
                    println!("position updated");
                } else {
                    commands.spawn((
                        SavedPosition,
                        Transform::from_xyz(
                            data.avatar.position[0],
                            data.avatar.position[1],
                            data.avatar.position[2],
                        ),
                    ));
                    println!("new position saved");
                }
            }
        }
    }
}

fn gizmo(
    mut gizmos: Gizmos,
    mut events: EventReader<MumbleLinkEvent>,
    mut mumbledata: ResMut<MumbleData>,
    query: Query<&Transform, With<SavedPosition>>,
) {
    let position = Vec3::new(0.0, 100., 0.0);
    gizmos.sphere(position, Quat::default(), 1.0, basic::RED);

    for event in events.read() {
        if let MumbleLinkEvent::MumbleLinkData(data) = event {
            mumbledata.0 = Some(data.clone());
        }
    }

    if let Ok(saved_pos) = query.get_single() {
        let pos = saved_pos.translation;
        gizmos.sphere(pos, Quat::default(), 1.0, basic::FUCHSIA);
    }

    if let Some(data) = &mumbledata.0 {
        let player = Vec3::new(
            data.avatar.position[0],
            data.avatar.position[1],
            data.avatar.position[2],
        );
        gizmos.arrow(player, player + Vec3::X, basic::RED);
        gizmos.arrow(player, player + Vec3::Y, basic::GREEN);
        gizmos.arrow(player, player + Vec3::Z, basic::BLUE);
    }
}

fn input(input: Res<ButtonInput<KeyCode>>, mut mumble_link_event: EventWriter<MumbleLinkEvent>) {
    if input.just_pressed(KeyCode::Escape) {
        mumble_link_event.send(MumbleLinkEvent::Toggle);
    }
}
