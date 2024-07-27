use bevy::prelude::*;
use bevy_console::{clap::Parser, AddConsoleCommand, ConsoleCommand, ConsolePlugin};
use clap::{Subcommand, ValueEnum};

use crate::{
    link::MapId,
    marker::MarkerEvent,
    ui::compass::marker::{CompassMarker, ShowOnCompass},
    MarkerPacks, UiEvent,
};

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ConsolePlugin);
        app.add_console_command::<UnloadAllCommand, _>(unload_all_command);
        app.add_console_command::<MapIdCommand, _>(mapid_command);
        app.add_console_command::<PacksCommand, _>(packs_command);
        app.add_console_command::<SetupCommand, _>(setup_command);
        app.add_console_command::<AddCommand, _>(add_command);
        app.add_console_command::<DeleteCommand, _>(delete_command);
    }
}

/// List all the markers available
#[derive(Parser, ConsoleCommand)]
#[command(name = "unloadall")]
struct UnloadAllCommand;

fn unload_all_command(
    mut log: ConsoleCommand<UnloadAllCommand>,
    mut events: EventWriter<MarkerEvent>,
) {
    if let Some(Ok(UnloadAllCommand)) = log.take() {
        events.send(MarkerEvent::HideAllMarkers);
        log.reply_ok("Unloaded all markers");
    }
}

/// Override the current map_id
#[derive(Parser, ConsoleCommand)]
#[command(name = "mapid")]
struct MapIdCommand {
    new_id: u32,
}

fn mapid_command(mut log: ConsoleCommand<MapIdCommand>, mut commands: Commands) {
    if let Some(Ok(MapIdCommand { new_id })) = log.take() {
        commands.insert_resource(MapId(new_id));
        log.reply_ok(format!("Loaded map_id {new_id}"));
    }
}

/// List the loaded marker packs
#[derive(Parser, ConsoleCommand)]
#[command(name = "packs")]
struct PacksCommand;

fn packs_command(mut log: ConsoleCommand<PacksCommand>, packs: Res<MarkerPacks>) {
    if let Some(Ok(PacksCommand)) = log.take() {
        for id in packs.keys() {
            log.reply(&id.0);
        }
        log.reply_ok("");
    }
}

#[derive(ValueEnum, Clone, Copy)]
enum System {
    Compass,
}

/// Initialize some systems to some default value
#[derive(Parser, ConsoleCommand)]
#[command(name = "setup")]
struct SetupCommand {
    system: System,
}

fn setup_command(mut log: ConsoleCommand<SetupCommand>, mut ui_events: EventWriter<UiEvent>) {
    if let Some(Ok(SetupCommand { system })) = log.take() {
        match system {
            System::Compass => {
                ui_events.send(UiEvent::CompassSize(UVec2 { x: 362, y: 362 }));
            }
        }
        log.reply_ok("");
    }
}

/// Add things.
#[derive(Parser, ConsoleCommand)]
#[command(name = "add")]
struct AddCommand {
    #[command(subcommand)]
    kind: Add,
}

#[derive(Subcommand, Clone)]
enum Add {
    Marker { x: f32, y: f32 },
}

#[derive(Component)]
struct CreatedByConsole;

fn add_command(
    mut log: ConsoleCommand<AddCommand>,
    mut commands: Commands,
    packs: Res<MarkerPacks>,
) {
    if let Some(Ok(AddCommand { kind })) = log.take() {
        match kind {
            Add::Marker { x, y } => {
                let icon = packs.values().next().unwrap().get_images().next().unwrap();

                commands.spawn((
                    CreatedByConsole,
                    Transform::from_translation(Vec3::new(x, 0.0, y)), //
                    ShowOnCompass(icon.clone()),
                ));
            }
        }
        log.reply_ok("");
    }
}

/// Delete things.
#[derive(Parser, ConsoleCommand)]
#[command(name = "delete")]
struct DeleteCommand {
    #[command(subcommand)]
    kind: Delete,
}
#[derive(Subcommand, Clone)]
enum Delete {
    Markers,
}

fn delete_command(
    mut log: ConsoleCommand<DeleteCommand>,
    mut commands: Commands,
    mut q_items: Query<Entity, With<CreatedByConsole>>,
    q_markers: Query<(Entity, &CompassMarker)>,
) {
    if let Some(Ok(DeleteCommand { kind })) = log.take() {
        match kind {
            Delete::Markers => {
                for item in &q_items
                    .transmute_lens_filtered::<Entity, With<ShowOnCompass>>()
                    .query()
                {
                    commands.entity(item).despawn_recursive();
                    if let Some((entity, _)) = q_markers.iter().find(|(_, marker)| marker.0 == item)
                    {
                        commands.entity(entity).despawn_recursive();
                    } else {
                        log.reply("Could not find associated compass marker.");
                    }
                }
            }
        }
        log.ok();
    }
}
