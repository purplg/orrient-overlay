mod theme;
pub(super) use theme::*;

use orrient_pathing::prelude::*;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use sickle_ui::prelude::*;

use crate::UiEvent;

use super::downloads::DownloadView;
use super::installed::tooltip::UiToolTipExt as _;
use super::installed::InstalledView;

#[derive(Event)]
pub(super) enum MarkerWindowEvent {
    SetRootColumn,
    SetColumn {
        column_id: usize,
        full_id: FullMarkerId,
    },
}

#[derive(Component)]
pub struct OrrientMenuItem(pub MarkerEvent);

#[derive(Component)]
pub(super) struct MarkerWindow;

#[derive(Component)]
pub(super) struct MarkerItem {
    pub id: FullMarkerId,
    pub tip: Option<String>,
    pub description: Option<String>,
}

fn spawn_ui(mut commands: Commands) {
    commands
        .ui_builder(UiRoot)
        .floating_panel(
            FloatingPanelConfig {
                title: Some("Markers".into()),
                ..default()
            },
            FloatingPanelLayout {
                size: (1000., 1080.).into(),
                position: Some((100., 100.).into()),
                ..default()
            },
            |parent| {
                parent.menu_bar(|parent| {
                    parent.menu(
                        MenuConfig {
                            name: "Menu".into(),
                            ..default()
                        },
                        |parent| {
                            parent
                                .menu_item(MenuItemConfig {
                                    name: "Hide all markers".into(),
                                    ..default()
                                })
                                .insert(OrrientMenuItem(MarkerEvent::DisableAll));
                        },
                    );
                });

                parent
                    .spawn(MarkerListView::frame())
                    .tab_container(|parent| {
                        parent.add_tab("Markers".into(), |parent| {
                            parent.spawn(InstalledView::frame());
                        });
                        parent.add_tab("Downloads".into(), |parent| {
                            parent.spawn(DownloadView::frame());
                        });
                    });
            },
        )
        .insert(MarkerWindow)
        .enable_tooltip();
}

fn toggle_show_ui(
    mut events: EventReader<UiEvent>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut ui: Query<&mut FloatingPanelConfig, With<MarkerWindow>>,
) {
    for event in events.read() {
        match event {
            UiEvent::ToggleUI => {
                let mut window = window.single_mut();
                if window.cursor.hit_test {
                    window.cursor.hit_test = false;
                    ui.single_mut().folded = true;
                    info!("UI disabled");
                } else {
                    window.cursor.hit_test = true;
                    ui.single_mut().folded = false;
                    info!("UI enabled");
                }
            }
            UiEvent::CloseUi => {
                let mut window = window.single_mut();
                window.cursor.hit_test = false;
                ui.single_mut().folded = true;
                info!("UI disabled");
            }
            _ => {}
        }
    }
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MarkerWindowEvent>();
        app.add_plugins(theme::Plugin);

        app.add_systems(Startup, spawn_ui);
        app.add_systems(Update, toggle_show_ui);
    }
}
