use bevy::prelude::*;

use orrient_core::prelude::*;

use orrient_api::{Request, RequestComplete, RequestQueue, Response};

#[derive(Resource, Clone)]
pub(super) struct MapBounds {
    pub(super) map: Rect,
    pub(super) continent: Rect,
}

fn update_bounds(mut commands: Commands, map_id: Res<MapId>, mut queue: ResMut<RequestQueue>) {
    commands.remove_resource::<MapBounds>();
    queue.fetch(Request::Map(map_id.0));
}

fn api_response_system(
    mut commands: Commands,
    mut api_events: EventReader<RequestComplete>,
    map_id: Res<MapId>,
) {
    for event in api_events.read() {
        if let RequestComplete(Response::Map(map)) = event {
            if map_id.0 == map.id {
                commands.insert_resource(MapBounds {
                    map: Rect {
                        min: Vec2::new(map.map_rect.bottom_left.x, map.map_rect.bottom_left.y),
                        max: Vec2::new(map.map_rect.top_right.x, map.map_rect.top_right.y),
                    },
                    continent: Rect {
                        min: Vec2::new(
                            map.continent_rect.top_left.x,
                            map.continent_rect.top_left.y,
                        ),
                        max: Vec2::new(
                            map.continent_rect.bottom_right.x,
                            map.continent_rect.bottom_right.y,
                        ),
                    },
                });
            }
        }
    }
}

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_bounds.run_if(resource_exists_and_changed::<MapId>),
        );
        app.add_systems(
            Update,
            api_response_system.run_if(on_event::<RequestComplete>()),
        );
    }
}
