use bevy::prelude::*;
use bevy_console::{clap::Parser, AddConsoleCommand, ConsoleCommand, ConsolePlugin};

use crate::{link::MapId, MarkerPacks, UiEvent};

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ConsolePlugin);
        app.add_console_command::<UnloadAllCommand, _>(unload_all_command);
        app.add_console_command::<MapIdCommand, _>(mapid_command);
        app.add_console_command::<PacksCommand, _>(packs_command);
    }
}

/// List all the markers available
#[derive(Parser, ConsoleCommand)]
#[command(name = "unloadall")]
struct UnloadAllCommand;

fn unload_all_command(mut log: ConsoleCommand<UnloadAllCommand>, mut events: EventWriter<UiEvent>) {
    if let Some(Ok(UnloadAllCommand)) = log.take() {
        events.send(UiEvent::UnloadAllMarkers);
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
