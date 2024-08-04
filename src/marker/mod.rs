pub mod poi;
pub mod trail;

use bevy::{prelude::*, utils::HashSet};

use crate::FullMarkerId;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MarkerEvent>();
        app.add_plugins(poi::Plugin);
        app.add_plugins(trail::Plugin);
    }
}

#[derive(Resource, Clone, Deref, DerefMut, Debug, Default)]
pub struct LoadedMarkers(pub HashSet<FullMarkerId>);

#[derive(Event, Clone, Debug)]
pub enum MarkerEvent {
    Show(FullMarkerId),
    Hide(FullMarkerId),
    HideAll,
}
