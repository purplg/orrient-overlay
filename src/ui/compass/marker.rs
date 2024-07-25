use bevy::color::palettes;
use bevy::prelude::*;
use bevy::utils::HashMap;
use lazy_static::lazy_static;
use sickle_ui::prelude::UiContainerExt as _;
use sickle_ui::ui_builder::UiBuilder;
use sickle_ui::ui_builder::UiBuilderExt;
use sickle_ui::ui_style::manual::SetAbsolutePositionExt;

use crate::link::MapId;

use super::window::CompassWindow;

mod mapid {
    pub const QUEENSDALE: u32 = 15;
    pub const MISTLOCK: u32 = 1206;
}

// TODO Replace with API.
lazy_static! {
    static ref MAP: HashMap<u32, Rect> = {
        let mut m = HashMap::new();
        m.insert(
            mapid::QUEENSDALE,
            Rect {
                min: Vec2::new(-43008., -27648.),
                max: Vec2::new(43008., 30720.),
            },
        );
        m.insert(
            mapid::MISTLOCK,
            Rect {
                min: Vec2::new(-12288., -12288.),
                max: Vec2::new(12288., 12288.),
            },
        );
        m
    };
    static ref CONTINENT: HashMap<u32, Rect> = {
        let mut m = HashMap::new();
        m.insert(
            mapid::QUEENSDALE,
            Rect {
                min: Vec2::new(42624., 28032.),
                max: Vec2::new(46208., 30464.),
            },
        );
        m.insert(
            mapid::MISTLOCK,
            Rect {
                min: Vec2::new(46368., 33520.),
                max: Vec2::new(48416., 35568.),
            },
        );
        m
    };
}

fn get_bounds(map_id: &u32) -> Option<Bounds> {
    Some(Bounds {
        map: MAP.get(map_id)?,
        continent: CONTINENT.get(map_id)?,
    })
}

struct Bounds<'a> {
    map: &'a Rect,
    continent: &'a Rect,
}

#[derive(Component)]
pub struct CompassMarker(pub Entity);

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
    map_id: Res<MapId>,
) {
    let compass_window = q_compass.single();
    for (entity, marker) in &mut q_compass_markers {
        let Ok(transform) = q_world_markers.get(marker.0) else {
            warn!("World marker not found.");
            continue;
        };
        // TODO Account for compass rotation
        let world_position = transform.translation.xz() * METERS_TO_INCHES;
        let Some(Bounds { map, continent }) = get_bounds(&map_id.0) else {
            warn!("No bounds defined for map_id: {}", map_id.0);
            return;
        };

        let d = world_position - map.min;
        let px = d.x / map.width();
        let py = d.y / map.height();
        let x = continent.min.x + px * continent.width();
        let y = continent.min.y + py * continent.height();

        commands
            .ui_builder(entity)
            .style()
            .absolute_position(compass_window.clamp(Vec2 { x, y }));
    }
}

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, spawn_markers);
        app.add_systems(Update, position_system);
    }
}
