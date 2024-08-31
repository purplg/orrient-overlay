use bevy::color::palettes;
use bevy::prelude::*;
use bevy_mod_billboard::BillboardTextureHandle;
use orrient_core::prelude::MapId;
use orrient_pathing::marker::poi::PoiMarker;
use sickle_ui::prelude::UiContainerExt as _;
use sickle_ui::ui_builder::UiBuilder;
use sickle_ui::ui_builder::UiBuilderExt;
use sickle_ui::ui_style::manual::SetAbsolutePositionExt;

use super::map_bounds::MapBounds;
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
    trigger: Trigger<OnAdd, PoiMarker>,
    mut commands: Commands,
    q_images: Query<&BillboardTextureHandle>,
    q_compass: Query<Entity, With<CompassWindow>>,
) {
    if let Ok(billboard) = q_images.get(trigger.entity()) {
        commands
            .ui_builder(q_compass.single())
            .compass_marker(trigger.entity(), billboard.0.clone());
    } else {
        warn!("No icon for compass marker found.")
    }
}

fn despawn_marker(
    trigger: Trigger<OnRemove, PoiMarker>,
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
    q_poi_markers: Query<&Transform, With<PoiMarker>>,
    mut q_compass_markers: Query<(Entity, &CompassMarker)>,
    q_compass: Query<&CompassWindow>,
    bounds: Res<MapBounds>,
) {
    let compass_window = q_compass.single();
    for (entity, marker) in &mut q_compass_markers {
        let Ok(transform) = q_poi_markers.get(marker.0) else {
            warn!("World marker not found.");
            continue;
        };
        // TODO Account for compass rotation
        let world_position = transform.translation.xz() * METERS_TO_INCHES;

        let d = world_position - bounds.map.min;
        let px = d.x / bounds.map.width();
        let py = d.y / bounds.map.height();
        let x = bounds.continent.min.x + px * bounds.continent.width();
        let y = bounds.continent.min.y + py * bounds.continent.height();

        commands
            .ui_builder(entity)
            .style()
            .absolute_position(compass_window.clamp(Vec2 { x, y }));
    }
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            position_system.run_if(resource_exists::<MapId>.and_then(resource_exists::<MapBounds>)),
        );
        app.observe(spawn_marker);
        app.observe(despawn_marker);
    }
}
