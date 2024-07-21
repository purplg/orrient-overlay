use bevy::color::palettes;
use bevy::prelude::*;
use sickle_ui::prelude::UiContainerExt as _;
use sickle_ui::ui_builder::UiBuilder;
use sickle_ui::ui_builder::UiBuilderExt;
use sickle_ui::ui_style::manual::SetAbsolutePositionExt;

use crate::MarkerPacks;
use crate::UiEvent;

use super::window::CompassWindow;

const TYRIA: Rect = Rect {
    min: Vec2::new(9856., 11648.),
    max: Vec2::new(13440., 14080.),
};

const QUEENSDALE: Rect = Rect {
    min: Vec2::new(-43008., -27648.),
    max: Vec2::new(43008., 30720.),
};

#[derive(Component)]
struct CompassMarker;

impl CompassMarker {
    fn frame() -> impl Bundle {
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Px(16.),
                height: Val::Px(16.),
                ..default()
            },
            background_color: palettes::basic::RED.into(),
            ..default()
        }
    }
}

#[derive(Component, Default)]
struct CompassPosition(Vec2);

pub trait UiCompassMarkerExt {
    fn compass_marker(&mut self);
}

impl UiCompassMarkerExt for UiBuilder<'_, Entity> {
    fn compass_marker(&mut self) {
        self.container(CompassMarker::frame(), |parent| {}).insert((
            CompassMarker, //
            CompassPosition::default(),
        ));
    }
}

impl CompassPosition {
    /// Convert the world position to screen space position relative
    /// to the compass widget.
    fn to_compass(&self, continent: Rect, map: Rect, coords: Vec2) -> Vec2 {
        let x = continent.min.x + (1. * coords.x - map.min.x) / (map.width()) * (continent.width());
        let y =
            continent.min.y + (1. * coords.y - map.min.y) / (map.height()) * (continent.height());

        Vec2 { x, y }
    }
}

fn player_test_update(
    mut q_compass_markers: Query<&mut CompassPosition>,
    mut events: EventReader<UiEvent>,
) {
    for event in events.read() {
        match event {
            UiEvent::PlayerPosition(pos) => {
                let mut marker_position = q_compass_markers.single_mut();
                // marker_position.0 = Vec2 { x: pos.x, y: pos.y };
                marker_position.0 = Vec2::new(43681.336, 28713.953);
            }
            _ => {}
        }
    }
}

fn spawn_system(mut commands: Commands, mut events: EventReader<UiEvent>, packs: Res<MarkerPacks>) {
    for event in events.read() {
        match event {
            UiEvent::LoadMarker(full_id) => {
                let Some(pack) = packs.get(&full_id.pack_id) else {
                    continue;
                };

                let Some(pois) = pack.get_pois(&full_id.marker_id) else {
                    continue;
                };

                for poi in pois {
                    commands.spawn((
                        CompassMarker,
                        CompassPosition(Vec2::ZERO),
                        ImageBundle {
                            style: Style {
                                width: Val::Px(16.),
                                height: Val::Px(16.),
                                ..default()
                            },
                            background_color: palettes::basic::RED.into(),
                            ..default()
                        },
                    ));
                }
            }
            _ => {}
        }
    }
}

fn position_system(
    mut commands: Commands,
    mut q_compass_markers: Query<(Entity, &CompassPosition)>,
    q_compass: Query<&CompassWindow>,
) {
    let compass_window = q_compass.single();
    for (entity, position) in &mut q_compass_markers {
        commands
            .ui_builder(entity)
            .style()
            .absolute_position(compass_window.clamp(position.0));
    }
}

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, position_system);
        app.add_systems(Update, player_test_update.run_if(on_event::<UiEvent>()));
        // app.add_systems(Update, spawn_system.run_if(on_event::<UiEvent>()));
    }
}
