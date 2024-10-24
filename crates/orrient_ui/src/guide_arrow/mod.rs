use std::time::Duration;

use bevy::{color::palettes, prelude::*, time::common_conditions::on_timer};
use orrient_core::prelude::Player;
use orrient_pathing::prelude::*;

#[derive(Resource, Default)]
struct Closest(Option<Vec3>);

fn draw(mut gizmos: Gizmos, player: Query<&Transform, With<Player>>, closest: Res<Closest>) {
    let Some(closest) = closest.0 else {
        return;
    };

    let Ok(player) = player.get_single() else {
        return;
    };
    let pos = player.translation;

    let dir = (closest - pos).normalize();
    gizmos.arrow(pos, pos + dir, palettes::basic::RED);
}

fn update(
    mut closest: ResMut<Closest>,
    player: Query<&Transform, With<Player>>,
    map_markers: Res<MapMarkers>,
    enabled_markers: Res<EnabledMarkers>,
    packs: Res<MarkerPacks>,
) {
    let Ok(player) = player.get_single() else {
        return;
    };
    let pos = player.translation;
    closest.0 = enabled_markers
        .intersection(&map_markers)
        .filter_map(|full_id| {
            packs.get(&full_id.pack_id).and_then(|pack| {
                pack.find_by_name(full_id.marker_name.clone())
                    .and_then(|node_id| pack.get(node_id))
                    .map(|node| node.data())
                    .map(|marker| marker.pois.clone())
            })
        })
        .flatten()
        .filter_map(|poi| poi.position)
        .reduce(|a, b| {
            if a.distance(pos) < b.distance(pos) {
                a
            } else {
                b
            }
        });
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Closest>();
        app.insert_resource(Closest(Some(Vec3::ZERO)));
        app.add_systems(Update, draw);
        app.add_systems(Update, update.run_if(on_timer(Duration::from_secs(1))));
    }
}
