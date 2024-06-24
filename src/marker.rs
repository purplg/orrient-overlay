use std::collections::HashMap;

use bevy::prelude::*;
use marker::trail;

use crate::OrrientEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(PreUpdate, load_marker.run_if(on_event::<OrrientEvent>()));
        app.add_systems(Update, load_trail_system.run_if(on_event::<OrrientEvent>()));
        app.add_systems(Update, load_pois_system.run_if(on_event::<OrrientEvent>()));
    }
}

fn setup(mut events: EventWriter<OrrientEvent>) {
    events.send(OrrientEvent::LoadMarkers(
        "tw_lws03e05_draconismons.xml".into(),
    ));
}

fn load_marker(mut commands: Commands, mut events: EventReader<OrrientEvent>) {
    for event in events.read() {
        if let OrrientEvent::LoadMarkers(filename) = event {
            if let Ok(markers) = marker::read(
                &dirs::config_dir()
                    .unwrap()
                    .join("orrient")
                    .join("markers")
                    .join(filename),
            ) {
                commands.insert_resource(MarkerSet(markers));
            }
        }
    }
}

#[derive(Resource)]
pub struct Trail(pub Vec<Vec3>);

fn load_trail_system(
    mut commands: Commands,
    mut orrient_events: EventReader<OrrientEvent>,
    data: Res<MarkerSet>,
) {
    for event in orrient_events.read() {
        if let OrrientEvent::LoadTrail(trail_id) = event {
            let path: Vec<&str> = trail_id.split(".").collect();
            if let Some(marker) = &data.get_path(path) {
                if let Some(trail) = &marker.trail_file {
                    let trail_path = dirs::config_dir()
                        .unwrap()
                        .join("orrient")
                        .join("markers")
                        .join(&trail);
                    if let Ok(trail) = trail::from_file(trail_path.as_path()) {
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
                }
            }
        }
    }
}

#[derive(Resource)]
pub struct POIs(pub Vec<Vec3>);

fn load_pois_system(
    mut commands: Commands,
    mut orrient_events: EventReader<OrrientEvent>,
    data: Res<MarkerSet>,
) {
    for event in orrient_events.read() {
        if let OrrientEvent::LoadTrail(trail_id) = event {
            let path = trail_id.split(".").collect();
            if let Some(marker) = data.get_path(path) {
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
    }
}

#[derive(Resource, Clone, Deref, Debug)]
pub struct MarkerSet(pub marker::Markers);
