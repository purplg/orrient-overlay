use std::ffi::OsStr;

use bevy::{prelude::*, window::PrimaryWindow};
use marker::MarkerKind;
use sickle_ui::{
    ui_builder::{UiBuilder, UiBuilderExt, UiRoot},
    ui_style::*,
    widgets::{container::UiContainerExt, prelude::*, scroll_view::UiScrollViewExt},
    SickleUiPlugin,
};

use crate::{marker::MarkerTree, UiEvent};

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SickleUiPlugin);

        app.add_event::<CheckboxEvent>();

        app.add_systems(Startup, setup);
        app.add_systems(Update, toggle_show_ui);
        app.add_systems(Update, checkbox);
        app.add_systems(Update, checkbox_events);
        app.add_systems(Update, menu_interaction);
        app.add_systems(Update, show_file_open.run_if(on_event::<UiEvent>()));
        app.add_systems(Update, hide_file_open.run_if(on_event::<UiEvent>()));
        app.add_systems(
            Update,
            (remove_markers, show_markers)
                .chain()
                .run_if(resource_exists_and_changed::<MarkerTree>),
        );
    }
}

#[derive(Component)]
struct MarkerView;

#[derive(Component)]
struct FileBrowser;

#[derive(Component)]
struct OrrientMenuItem(UiEvent);

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
            NodeBundle::default(), //
            TargetCamera(camera),
        ),
        |container| {
            container.floating_panel(
                FloatingPanelConfig {
                    title: Some("Markers".into()),
                    ..default()
                },
                FloatingPanelLayout {
                    size: (1920., 1080.).into(),
                    position: Some((100., 100.).into()),
                    ..default()
                },
                |panel| {
                    panel
                        .spawn((
                            NodeBundle::default(),
                            MarkerView, //
                        ))
                        .style()
                        .width(Val::Percent(100.))
                        .height(Val::Percent(100.));
                },
            );
        },
    );
}

#[derive(Component)]
struct MarkerItem(String);

fn tree_item(
    item: &marker::MarkerTreeItem<'_>,
    parent: &mut UiBuilder<'_, '_, '_, Entity>,
    markers: &MarkerTree,
) {
    parent
        .row(|parent| {
            parent
                .checkbox(Some(""), false) //
                .insert(MarkerItem(item.id.to_string()))
                .style()
                .width(Val::Px(42.));

            parent
                .foldable(&item.marker.label, true, false, |parent| {
                    for subitem in markers.iter(item.id) {
                        match subitem.marker.kind {
                            MarkerKind::Category => {
                                tree_item(&subitem, parent, markers);
                            }
                            MarkerKind::Leaf => {
                                let label = subitem.marker.label.clone();
                                parent
                                    .checkbox(Some(label), false)
                                    .insert(MarkerItem(subitem.id.to_string()))
                                    .style()
                                    .width(Val::Percent(100.))
                                    .left(Val::Px(10. * subitem.depth as f32));
                            }
                            MarkerKind::Separator => {
                                parent
                                    .label(LabelConfig::from(&subitem.marker.label))
                                    .style()
                                    .width(Val::Percent(100.))
                                    .background_color(Color::BLACK);
                            }
                        }
                    }
                })
                .style()
                .padding(UiRect::vertical(Val::Px(3.)));
        })
        .style()
        .align_items(AlignItems::FlexStart)
        .border(UiRect::left(Val::Px(1.)))
        .border_color(Color::rgba(0., 0., 0., 0.5))
        .width(Val::Percent(100.));
}

fn show_markers(
    mut commands: Commands,
    markers: Res<MarkerTree>,
    query: Query<Entity, With<MarkerView>>,
) {
    commands
        .ui_builder(query.single())
        .column(|parent| {
            parent.menu_bar(|parent| {
                parent.menu(
                    MenuConfig {
                        name: "File".into(),
                        ..default()
                    },
                    |parent| {
                        parent
                            .menu_item(MenuItemConfig {
                                name: "Open markers...".into(),
                                ..default()
                            })
                            .insert(OrrientMenuItem(UiEvent::ShowMarkerBrowser));
                    },
                );
            });

            parent.scroll_view(None, |scroll_view| {
                if let Some(item) = markers.root() {
                    tree_item(&item, scroll_view, &markers);
                }
            });
        })
        .style()
        .width(Val::Percent(100.));
}

fn remove_markers(mut commands: Commands, query: Query<Entity, With<MarkerView>>) {
    commands.entity(query.single()).despawn_descendants();
}

#[derive(Event, Debug)]
enum CheckboxEvent {
    Enable(String),
    Disable(String),
}

impl CheckboxEvent {
    fn id(&self) -> &str {
        match self {
            CheckboxEvent::Enable(id) => id,
            CheckboxEvent::Disable(id) => id,
        }
    }

    fn enabled(&self) -> bool {
        match self {
            CheckboxEvent::Enable(_) => true,
            CheckboxEvent::Disable(_) => false,
        }
    }
}

fn checkbox(
    query: Query<(&Checkbox, &MarkerItem), Changed<Checkbox>>,
    mut checkbox_events: EventWriter<CheckboxEvent>,
    mut ui_events: EventWriter<UiEvent>,
    markers: Res<MarkerTree>,
) {
    for (checkbox, item) in query.iter() {
        if checkbox.checked {
            ui_events.send(UiEvent::LoadMarker(item.0.to_string()));
            checkbox_events.send_batch(
                markers
                    .iter(&item.0)
                    .map(|item| CheckboxEvent::Enable(item.id.to_string())),
            );
        } else {
            ui_events.send(UiEvent::UnloadMarker(item.0.to_string()));
            checkbox_events.send_batch(
                markers
                    .iter(&item.0)
                    .map(|item| CheckboxEvent::Disable(item.id.to_string())),
            );
        }
    }
}

fn checkbox_events(
    mut query: Query<(&mut Checkbox, &MarkerItem)>,
    mut checkbox_events: EventReader<CheckboxEvent>,
) {
    for event in checkbox_events.read() {
        let event_id = event.id().to_string();

        if let Some(mut checkbox) = query.iter_mut().find_map(|(checkbox, item)| {
            if item.0 == event_id {
                Some(checkbox)
            } else {
                None
            }
        }) {
            checkbox.checked = event.enabled();
        }
    }
}

fn toggle_show_ui(
    mut commands: Commands,
    mut events: EventReader<UiEvent>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    ui: Query<Entity, With<MarkerView>>,
) {
    for event in events.read() {
        if let UiEvent::ToggleUI = event {
            let mut window = window.single_mut();
            let visible = !window.cursor.hit_test;
            if visible {
                window.cursor.hit_test = true;
                commands.entity(ui.single()).insert(Visibility::Visible);
                info!("UI enabled");
            } else {
                window.cursor.hit_test = false;
                commands.entity(ui.single()).insert(Visibility::Hidden);
                info!("UI disabled");
            }
        }
    }
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
    for event in events.read() {
        match event {
            UiEvent::LoadMarkers(_) => break,
            _ => return,
        }
    }

    let Ok(browser) = query.get_single() else {
        return;
    };

    commands.entity(browser).despawn_recursive();
}

fn menu_interaction(
    query: Query<(&MenuItem, &OrrientMenuItem), Changed<MenuItem>>,
    mut events: EventWriter<UiEvent>,
) {
    for (menu_item, orrient_menu_item) in &query {
        if menu_item.interacted() {
            events.send(orrient_menu_item.0.clone());
        }
    }
}
