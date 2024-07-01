use std::ffi::OsStr;

use bevy::prelude::*;
use bevy_console::{clap::Parser, AddConsoleCommand, ConsoleCommand, ConsolePlugin};

use crate::UiEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ConsolePlugin);
        app.add_console_command::<ListMarkers, _>(list_command);
        app.add_console_command::<LoadMarkers, _>(load_command);
    }
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "ls")]
/// List all the markers available
struct ListMarkers;

fn list_command(mut log: ConsoleCommand<ListMarkers>) {
    if let Some(Ok(ListMarkers)) = log.take() {
        let dir = &dirs::config_dir().unwrap().join("orrient").join("markers");
        let iter = std::fs::read_dir(dir).unwrap();

        for item in iter
            .filter_map(Result::ok)
            .map(|file| file.path())
            .filter(|file| file.is_file())
            .filter(|file| Some(OsStr::new("xml")) == file.extension())
        {
            if let Some(filename) = item.file_name() {
                log.reply(format!("{}", filename.to_string_lossy()));
            }
        }
    }
}

#[derive(Parser, ConsoleCommand)]
#[command(name = "load")]
/// Load a set of markers.
struct LoadMarkers {
    file: String,
}

fn load_command(mut log: ConsoleCommand<LoadMarkers>, mut events: EventWriter<UiEvent>) {
    if let Some(Ok(LoadMarkers { file })) = log.take() {
        if dirs::config_dir()
            .unwrap()
            .join("orrient")
            .join("markers")
            .join(&file)
            .exists()
        {
            events.send(UiEvent::LoadMarkers(file));
            log.ok()
        } else {
            log.failed()
        }
    }
}
