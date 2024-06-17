mod link;
mod player;

use bevy::window::{CompositeAlphaMode, PrimaryWindow, WindowResolution};
use bevy::{
    prelude::*,
    window::{Cursor, WindowLevel},
};
use link::MumbleLinkEvent;

fn main() {
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

    app.add_plugins(link::Plugin);
    app.add_plugins(player::Plugin);

    app.add_systems(Startup, setup);
    app.add_systems(Update, camera_system);
    app.add_systems(
        Update,
        toggle_hittest_system.run_if(on_event::<MumbleLinkEvent>()),
    );
    app.add_systems(Update, input);

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        projection: Projection::Perspective(PerspectiveProjection {
            fov: 70.32_f32.to_radians(),
            ..default()
        }),
        ..default()
    });
}

fn camera_system(
    mut events: EventReader<MumbleLinkEvent>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera3d>>,
) {
    for event in events.read() {
        if let MumbleLinkEvent::Data(mumbledata) = event {
            let (mut transform, projection) = camera.single_mut();
            transform.translation = Vec3::new(
                mumbledata.camera.position[0],
                mumbledata.camera.position[1],
                -mumbledata.camera.position[2],
            );

            let Ok(forward) = Dir3::new(Vec3::new(
                mumbledata.camera.front[0],
                mumbledata.camera.front[1],
                mumbledata.camera.front[2],
            )) else {
                continue;
            };

            transform.rotation = Quat::IDENTITY;
            transform.rotate_x(forward.y.asin());
            transform.rotate_y(-forward.x.atan2(forward.z));

            if let Projection::Perspective(perspective) = projection.into_inner() {
                perspective.fov = mumbledata.identity.fov
            }
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

fn input(input: Res<ButtonInput<KeyCode>>, mut mumble_link_event: EventWriter<MumbleLinkEvent>) {
    if input.just_pressed(KeyCode::Escape) {
        mumble_link_event.send(MumbleLinkEvent::Toggle);
    }
}
