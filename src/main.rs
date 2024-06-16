use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use std::{env, thread};

use bevy::app::AppExit;
use bevy::color::palettes::basic::RED;
use bevy::input::keyboard::KeyboardInput;
use bevy::window::CompositeAlphaMode;
use bevy::{
    prelude::*,
    window::{Cursor, WindowLevel},
};

fn main() {
    // launch_gw();
    launch_bevy();
}

fn launch_gw() {
    let a = env::args().skip(1);
    let args = a.collect::<Vec<String>>();
    let args = ["-c".to_string(), args.join(" ")];
    let _handle = thread::spawn(|| {
        sleep(Duration::from_secs(5));
        if let Ok(child) = Command::new("sh".to_string()).args(args).spawn() {
            println!("child.id: {:?}", child.id());
            sleep(Duration::from_secs(5));
            if let Ok(output) = Command::new("xwininfo")
                .args(["-root", "-children"])
                .output()
            {
                let output = format!("{:?}", output).replace("\\n", "\n");
                println!("{}", output);
            }

            if let Ok(output) = Command::new("xprop")
                .args(["-id", "0x600003", "-set", "GAMESCOPE_EXTERNAL_OVERLAY", "1"])
                .output()
            {
                let output = format!("{:?}", output).replace("\\n", "\n");
                println!("{}", output);
            }

            // TODO Use mumblelink to get X11 Window ID from this process ID.
        }
    });
}

fn launch_bevy() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "GW2Orrient".to_string(),
            transparent: true,
            decorations: false,
            window_level: WindowLevel::AlwaysOnTop,
            composite_alpha_mode: CompositeAlphaMode::PreMultiplied,
            cursor: Cursor {
                // hit_test: false,
                ..default()
            },
            ..default()
        }),
        ..default()
    }));

    app.insert_resource(ClearColor(Color::NONE));

    app.add_systems(Startup, setup);
    app.add_systems(Update, gizmo);
    app.add_systems(Update, input.run_if(on_event::<KeyboardInput>()));

    app.run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn gizmo(mut gizmos: Gizmos) {
    gizmos.rect_2d(Vec2::ZERO, Rot2::default(), Vec2::splat(100.), RED);
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
