use bevy::prelude::*;
use marker::trail;

use crate::OrrientEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(
            PreUpdate,
            load_marker.run_if(resource_exists::<MarkerTree>.and_then(on_event::<OrrientEvent>())),
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
    data: Res<MarkerTree>,
) {
    for event in events.read() {
        let OrrientEvent::LoadMarker(marker_id) = event else {
            return;
        };

        let Some(id) = marker_id.split(".").last() else {
            warn!("Marker path not found: {}", marker_id);
            return;
        };

        let Some(marker) = &data.get(id) else {
            warn!("Marker ID not found: {}", id);
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

        commands.insert_resource(MarkerTree(markers));
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
        .join(trail);

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

    info!("Loaded trail with {} markers", trail.coordinates.len());
}

#[derive(Resource)]
pub struct POIs {
    pub long_id: String,
    pub positions: Vec<Vec3>,
}

fn load_pois_system(
    mut commands: Commands,
    mut orrient_events: EventReader<OrrientEvent>,
    data: Res<MarkerTree>,
) {
    for event in orrient_events.read() {
        let OrrientEvent::LoadMarker(marker_id) = event else {
            return;
        };

        info!("Loading POIs for {}", marker_id);

        let Some(id) = marker_id.split(".").last() else {
            warn!("Marker path not found: {}", marker_id);
            return;
        };

        let Some(marker) = &data.get(id) else {
            warn!("Marker ID not found: {}", id);
            return;
        };

        let pois: Vec<Vec3> = marker
            .pois
            .iter()
            .map(|poi| Vec3 {
                x: poi.x,
                y: poi.y,
                z: -poi.z,
            })
            .collect();

        info!("Loaded {} POIs.", pois.len());

        commands.insert_resource(POIs {
            long_id: marker_id.to_string(),
            positions: pois,
        });
    }
}

#[derive(Resource, Clone, Deref, Debug)]
pub struct Marker(pub marker::Marker);

#[derive(Resource, Clone, Deref, Debug)]
pub struct MarkerTree(pub marker::MarkerTree);
