use bevy::color::palettes;
use bevy::prelude::*;
use sickle_ui::prelude::UiContainerExt as _;
use sickle_ui::ui_builder::UiBuilder;
use sickle_ui::ui_builder::UiBuilderExt;
use sickle_ui::ui_style::manual::SetAbsolutePositionExt;

use crate::MarkerPacks;
use crate::UiEvent;

use super::window::CompassWindow;

const MAP_QUEENSDALE: Rect = Rect {
    min: Vec2::new(-43008., -27648.),
    max: Vec2::new(43008., 30720.),
};

const CONTINENT_QUEENSDALE: Rect = Rect {
    min: Vec2::new(42624., 28032.),
    max: Vec2::new(46208., 30464.),
};

#[derive(Component)]
struct CompassMarker(Entity);

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

#[derive(Component)]
pub struct ShowOnCompass(pub Handle<Image>);

pub trait UiCompassMarkerExt {
    fn compass_marker(&mut self, entity: Entity, image: Handle<Image>);
}

impl UiCompassMarkerExt for UiBuilder<'_, Entity> {
    fn compass_marker(&mut self, entity: Entity, icon: Handle<Image>) {
        let mut builder = self.container(CompassMarker::frame(), |parent| {
            parent.insert(ImageBundle {
                image: icon.into(),
                style: Style {
                    width: Val::Px(16.),
                    height: Val::Px(16.),
                    ..default()
                },
                ..default()
            });
        });
        builder.insert(CompassMarker(entity));
    }
}

impl ShowOnCompass {
    /// Convert the world position to screen space position relative
    /// to the compass widget.
    fn to_compass(&self, continent: Rect, map: Rect, coords: Vec2) -> Vec2 {
        let x = continent.min.x + (1. * coords.x - map.min.x) / (map.width()) * (continent.width());
        let y =
            continent.min.y + (1. * coords.y - map.min.y) / (map.height()) * (continent.height());

        Vec2 { x, y }
    }
}

fn spawn_markers(
    mut commands: Commands,
    q_compass_markers: Query<(Entity, &ShowOnCompass), Added<ShowOnCompass>>,
    q_compass: Query<Entity, With<CompassWindow>>,
) {
    for (entity, icon) in &q_compass_markers {
        commands
            .ui_builder(q_compass.single())
            .compass_marker(entity, icon.0.clone());
    }
}

const INCHES_TO_METERS: f32 = 0.0254;
const METERS_TO_INCHES: f32 = 39.3700787;

fn position_system(
    mut commands: Commands,
    q_world_markers: Query<&Transform, With<ShowOnCompass>>,
    mut q_compass_markers: Query<(Entity, &CompassMarker)>,
    q_compass: Query<&CompassWindow>,
) {
    let compass_window = q_compass.single();
    for (entity, marker) in &mut q_compass_markers {
        let Ok(transform) = q_world_markers.get(marker.0) else {
            warn!("World marker not found.");
            continue;
        };
        // TODO Account for compass rotation
        let world_position = transform.translation.xz() * METERS_TO_INCHES;

        let d = world_position - MAP_QUEENSDALE.min;
        let px = d.x / MAP_QUEENSDALE.width();
        let py = d.y / MAP_QUEENSDALE.height();

        let continent_x = CONTINENT_QUEENSDALE.min.x + px * CONTINENT_QUEENSDALE.width();
        let continent_y = CONTINENT_QUEENSDALE.min.y + py * CONTINENT_QUEENSDALE.height();

        commands
            .ui_builder(entity)
            .style()
            .absolute_position(compass_window.clamp(Vec2::new(continent_x, continent_y)));
    }
}

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_markers);
        app.add_systems(Update, position_system);
    }
}
