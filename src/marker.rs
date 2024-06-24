use bevy::prelude::*;
use marker::OverlayData;

use crate::OrrientEvent;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(PreUpdate, load_marker.run_if(on_event::<OrrientEvent>()));
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
                commands.insert_resource(MarkerSet { markers })
            }
        }
    }
}

#[derive(Resource, Clone, Deref, Debug)]
pub struct MarkerSet {
    pub markers: OverlayData,
}
