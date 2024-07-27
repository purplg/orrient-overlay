use bevy::prelude::*;
use bevy::utils::HashMap;

use crate::link::MapId;

pub const QUEENSDALE: u32 = 15;
pub const MISTLOCK: u32 = 1206;
pub const LORNARS_PASS: u32 = 27;

#[derive(Resource, Deref, DerefMut, Default)]
pub(super) struct MapBoundsCache(HashMap<u32, MapBounds>);

#[derive(Resource, Clone)]
pub(super) struct MapBounds {
    pub(super) map: Rect,
    pub(super) continent: Rect,
}

impl MapBoundsCache {
    pub(super) fn get(&self, map_id: &u32) -> Option<&MapBounds> {
        self.0.get(map_id)
    }
}

fn update_bounds(mut commands: Commands, map_id: Res<MapId>, cache: Res<MapBoundsCache>) {
    if let Some(bounds) = cache.get(&map_id.0) {
        commands.insert_resource(bounds.clone());
    }
}

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        let mut bounds = MapBoundsCache::default();
        bounds.insert(
            QUEENSDALE,
            MapBounds {
                map: Rect {
                    min: Vec2::new(-43008., -27648.),
                    max: Vec2::new(43008., 30720.),
                },
                continent: Rect {
                    min: Vec2::new(42624., 28032.),
                    max: Vec2::new(46208., 30464.),
                },
            },
        );
        bounds.insert(
            LORNARS_PASS,
            MapBounds {
                map: Rect {
                    min: Vec2::new(-21504., -58368.),
                    max: Vec2::new(21504., 58368.),
                },
                continent: Rect {
                    min: Vec2::new(50432., 29696.),
                    max: Vec2::new(52224., 34560.),
                },
            },
        );
        bounds.insert(
            MISTLOCK,
            MapBounds {
                map: Rect {
                    min: Vec2::new(-12288., -12288.),
                    max: Vec2::new(12288., 12288.),
                },
                continent: Rect {
                    min: Vec2::new(46368., 33520.),
                    max: Vec2::new(48416., 35568.),
                },
            },
        );
        app.insert_resource(bounds);
        app.add_systems(
            Update,
            update_bounds.run_if(resource_exists_and_changed::<MapId>),
        );
    }
}
