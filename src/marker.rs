use bevy::prelude::*;
use marker::trail;

use crate::OrrientEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(
            PreUpdate,
            load_marker.run_if(resource_exists::<MarkerSet>.and_then(on_event::<OrrientEvent>())),
        );
        app.add_systems(PreUpdate, load_markers.run_if(on_event::<OrrientEvent>()));
        app.add_systems(
            Update,
            load_trail_system.run_if(resource_exists_and_changed::<Marker>),
        );
        app.add_systems(
            Update,
            load_pois_system.run_if(resource_exists_and_changed::<Marker>),
        );
    }
}

fn setup(mut events: EventWriter<OrrientEvent>) {
    events.send(OrrientEvent::LoadMarkers(
        "tw_lws03e05_draconismons.xml".into(),
    ));
}

fn load_marker(
    mut commands: Commands,
    mut events: EventReader<OrrientEvent>,
    data: Res<MarkerSet>,
) {
    for event in events.read() {
        let OrrientEvent::LoadMarker(marker_id) = event else {
            return;
        };

        let path: Vec<&str> = marker_id.split(".").collect();
        let Some(marker) = &data.get_path(path) else {
            warn!("Error when trying to parse marker_id: {}", marker_id);
            return;
        };

        commands.insert_resource(Marker((*marker).clone()));
    }
}

fn load_markers(mut commands: Commands, mut events: EventReader<OrrientEvent>) {
    for event in events.read() {
        let OrrientEvent::LoadMarkers(filename) = event else {
            return;
        };

        let Ok(markers) = marker::read(
            &dirs::config_dir()
                .unwrap()
                .join("orrient")
                .join("markers")
                .join(filename),
        ) else {
            return;
        };

        commands.insert_resource(MarkerSet(markers));
    }
}

#[derive(Resource)]
pub struct Trail(pub Vec<Vec3>);

fn load_trail_system(mut commands: Commands, marker: Res<Marker>) {
    let Some(trail) = &marker.trail_file else {
        info!("No trail for this marker category.");
        return;
    };

    let trail_path = dirs::config_dir()
        .unwrap()
        .join("orrient")
        .join("markers")
        .join(&trail);

    let Ok(trail) = trail::from_file(trail_path.as_path()) else {
        error!("Error when loading trail file at: {:?}", trail_path);
        return;
    };

    commands.insert_resource(Trail(
        trail
            .coordinates
            .iter()
            .map(|coord| Vec3 {
                x: coord.x,
                y: coord.y,
                z: -coord.z,
            })
            .collect(),
    ));
    info!("Loaded trail");
}

#[derive(Resource)]
pub struct POIs(pub Vec<Vec3>);

fn load_pois_system(
    mut commands: Commands,
    mut orrient_events: EventReader<OrrientEvent>,
    data: Res<MarkerSet>,
) {
    for event in orrient_events.read() {
        let OrrientEvent::LoadMarker(trail_id) = event else {
            return;
        };

        let path = trail_id.split(".").collect();
        let Some(marker) = data.get_path(path) else {
            return;
        };

        commands.insert_resource(POIs(
            marker
                .pois
                .iter()
                .map(|poi| Vec3 {
                    x: poi.x,
                    y: poi.y,
                    z: -poi.z,
                })
                .collect(),
        ));
    }
}

#[derive(Resource, Clone, Deref, Debug)]
struct Marker(pub marker::Marker);

#[derive(Resource, Clone, Deref, Debug)]
pub struct MarkerSet(pub marker::Markers);
