mod debug_panel;
mod marker_list;

use bevy::{prelude::*, window::PrimaryWindow};

use debug_panel::UiDebugPanelExt as _;
use marker_list::window::UiMarkerWindowExt as _;
use sickle_ui::{
    ui_builder::{UiBuilderExt as _, UiRoot},
    widgets::prelude::*,
    SickleUiPlugin,
};
use std::ffi::OsStr;

use crate::UiEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SickleUiPlugin);

        app.add_plugins(marker_list::Plugin);
        app.add_plugins(debug_panel::Plugin);

        app.add_systems(Startup, setup);
        app.add_systems(Update, show_file_open.run_if(on_event::<UiEvent>()));
        app.add_systems(Update, hide_file_open.run_if(on_event::<UiEvent>()));
    }
}

#[derive(Component)]
struct OrrientMenuItem(pub UiEvent);

#[derive(Component)]
struct FileBrowser;

fn setup(mut commands: Commands) {
    let camera = commands
        .spawn(Camera3dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::rgba(0.0, 0.0, 0.0, 0.0)),
                ..default()
            },
            ..default()
        })
        .id();

    commands.ui_builder(UiRoot).container(
        (
            NodeBundle {
                background_color: Color::NONE.into(),
                ..default()
            },
            TargetCamera(camera),
        ),
        |container| {
            container.marker_window();
            container.debug_panel();
        },
    );
}

fn show_file_open(
    mut commands: Commands,
    query: Query<(), With<FileBrowser>>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut events: EventReader<UiEvent>,
) {
    // Wait for ShowFileBrowser event.
    if !events
        .read()
        .any(|event| matches!(event, UiEvent::ShowMarkerBrowser))
    {
        return;
    }

    // Already open
    if query.get_single().is_ok() {
        return;
    }

    let window = window.single();

    let root = commands
        .spawn((
            NodeBundle::default(), //
            FileBrowser,
        ))
        .id();

    let size = Vec2::new(800., 600.);
    let position = Vec2::new(window.width() * 0.5 - size.x * 0.5, 200.);
    commands.ui_builder(root).floating_panel(
        FloatingPanelConfig {
            title: Some("Open Markers".into()),
            draggable: false,
            resizable: false,
            foldable: false,
            closable: false,
            ..default()
        },
        FloatingPanelLayout {
            size,
            position: Some(position),
            ..default()
        },
        |parent| {
            let dir = &dirs::config_dir().unwrap().join("orrient").join("markers");
            let iter = std::fs::read_dir(dir).unwrap();
            for item in iter
                .filter_map(Result::ok)
                .map(|file| file.path())
                .filter(|file| file.is_file())
                .filter(|file| Some(OsStr::new("xml")) == file.extension())
            {
                let filename: String = item.file_name().unwrap().to_string_lossy().into();
                parent
                    .menu_item(MenuItemConfig {
                        name: filename.clone(),
                        ..default()
                    })
                    .insert(OrrientMenuItem(UiEvent::LoadMarkers(filename)));
            }
        },
    );
}

fn hide_file_open(
    mut commands: Commands,
    query: Query<Entity, With<FileBrowser>>,
    mut events: EventReader<UiEvent>,
) {
    if !events
        .read()
        .any(|event| matches!(event, UiEvent::LoadMarkers(_)))
    {
        return;
    }

    let Ok(browser) = query.get_single() else {
        return;
    };

    commands.entity(browser).despawn_recursive();
}
