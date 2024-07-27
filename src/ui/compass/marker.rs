use bevy::color::palettes;
use bevy::prelude::*;
use sickle_ui::prelude::UiContainerExt as _;
use sickle_ui::ui_builder::UiBuilder;
use sickle_ui::ui_builder::UiBuilderExt;
use sickle_ui::ui_style::manual::SetAbsolutePositionExt;

use crate::link::MapId;

use super::map_bounds::MapBounds;
use super::map_bounds::MapBoundsCache;
use super::window::CompassWindow;

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

fn spawn_marker(
    trigger: Trigger<OnAdd, ShowOnCompass>,
    mut commands: Commands,
    q_compass_markers: Query<&ShowOnCompass>,
    q_compass: Query<Entity, With<CompassWindow>>,
    bounds: Res<MapBoundsCache>,
    map_id: Res<MapId>,
) {
    if bounds.get(&map_id.0).is_some() {
        commands.ui_builder(q_compass.single()).compass_marker(
            trigger.entity(),
            q_compass_markers.get(trigger.entity()).unwrap().0.clone(),
        );
    } else {
        warn!("No bounds defined for map_id: {}", map_id.0);
    }
}

fn despawn_marker(
    trigger: Trigger<OnRemove, ShowOnCompass>,
    mut commands: Commands,
    q_compass_markers: Query<(Entity, &CompassMarker)>,
) {
    for (entity, _) in q_compass_markers
        .iter()
        .filter(|(_, compass_marker)| compass_marker.0 == trigger.entity())
    {
        commands.entity(entity).despawn_recursive();
    }
}

const METERS_TO_INCHES: f32 = 39.3700787;

fn position_system(
    mut commands: Commands,
    q_world_markers: Query<&Transform, With<ShowOnCompass>>,
    mut q_compass_markers: Query<(Entity, &CompassMarker)>,
    q_compass: Query<&CompassWindow>,
    bounds: Res<MapBoundsCache>,
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
        let Some(MapBounds { map, continent }) = bounds.get(&map_id.0) else {
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
        app.add_systems(Update, position_system.run_if(resource_exists::<MapId>));
        app.observe(spawn_marker);
        app.observe(despawn_marker);
    }
}
