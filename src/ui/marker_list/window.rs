use bevy::{prelude::*, window::PrimaryWindow};
use marker::MarkerKind;
use sickle_ui::{
    ui_builder::{UiBuilder, UiBuilderExt as _},
    ui_style::generated::*,
    widgets::prelude::*,
};

use crate::{marker::MarkerTree, ui::OrrientMenuItem, UiEvent};

use super::tooltip::UiToolTipExt as _;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CheckboxEvent>();
        app.add_systems(Update, menu_interaction);
        app.add_systems(Update, toggle_show_ui);
        app.add_systems(Update, checkbox);
        app.add_systems(Update, checkbox_events);
        app.add_systems(
            Update,
            (remove_markers, show_markers)
                .chain()
                .run_if(resource_exists_and_changed::<MarkerTree>),
        );
    }
}

#[derive(Component)]
pub(super) struct MarkerWindow;

#[derive(Component)]
struct MarkerList;

#[derive(Component)]
pub(super) struct MarkerItem {
    pub id: String,
    pub tip: Option<String>,
    pub description: Option<String>,
}

pub trait UiMarkerWindowExt {
    fn marker_window(&mut self);
}

impl UiMarkerWindowExt for UiBuilder<'_, Entity> {
    fn marker_window(&mut self) {
        self.floating_panel(
            FloatingPanelConfig {
                title: Some("Markers".into()),
                ..default()
            },
            FloatingPanelLayout {
                size: (1920., 1080.).into(),
                position: Some((100., 100.).into()),
                ..default()
            },
            |parent| {
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

                parent
                    .spawn((
                        NodeBundle::default(),
                        MarkerList, //
                    ))
                    .style()
                    .width(Val::Percent(100.))
                    .height(Val::Percent(100.));
            },
        )
        .insert(MarkerWindow);

        self.enable_tooltip();
    }
}

fn tree_item(item: &marker::Marker, parent: &mut UiBuilder<'_, Entity>, markers: &MarkerTree) {
    parent
        .row(|parent| {
            parent
                .checkbox(None, false) //
                .insert(MarkerItem {
                    id: item.id.to_string(),
                    tip: item.poi_tip.clone(),
                    description: item.poi_description.clone(),
                })
                .style()
                .width(Val::Px(42.));

            parent
                .foldable(&item.label, false, false, |parent| {
                    for subitem in markers.iter(&item.id) {
                        match subitem.kind {
                            MarkerKind::Category => {
                                tree_item(subitem, parent, markers);
                            }
                            MarkerKind::Leaf => {
                                let label = subitem.label.clone();
                                parent
                                    .checkbox(Some(label), false)
                                    .insert(MarkerItem {
                                        id: subitem.id.to_string(),
                                        tip: subitem.poi_tip.clone(),
                                        description: subitem.poi_description.clone(),
                                    })
                                    .style()
                                    .width(Val::Percent(100.))
                                    .left(Val::Px(10. * subitem.depth as f32));
                            }
                            MarkerKind::Separator => {
                                parent
                                    .label(LabelConfig::from(&subitem.label))
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
    query: Query<Entity, With<MarkerList>>,
) {
    commands
        .ui_builder(query.single())
        .column(|parent| {
            parent.scroll_view(None, |scroll_view| {
                for item in markers.roots() {
                    tree_item(item, scroll_view, &markers);
                }
            });
        })
        .style()
        .width(Val::Percent(100.));
}

fn remove_markers(mut commands: Commands, query: Query<Entity, With<MarkerList>>) {
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
            ui_events.send(UiEvent::LoadMarker(item.id.clone()));
            checkbox_events.send_batch(
                markers
                    .iter(&item.id)
                    .map(|item| CheckboxEvent::Enable(item.id.to_string())),
            );
        } else {
            ui_events.send(UiEvent::UnloadMarker(item.id.clone()));
            checkbox_events.send_batch(
                markers
                    .iter(&item.id)
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
            if item.id == event_id {
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
    mut events: EventReader<UiEvent>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut ui: Query<&mut FloatingPanelConfig, With<MarkerWindow>>,
) {
    for event in events.read() {
        if let UiEvent::ToggleUI = event {
            let mut window = window.single_mut();
            let visible = !window.cursor.hit_test;
            if visible {
                window.cursor.hit_test = true;
                ui.single_mut().folded = false;
                info!("UI enabled");
            } else {
                window.cursor.hit_test = false;
                ui.single_mut().folded = true;
                info!("UI disabled");
            }
        }
    }
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
