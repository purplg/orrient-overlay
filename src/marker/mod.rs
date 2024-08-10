pub mod poi;
pub mod trail;

use crate::prelude::*;
use bevy::utils::HashSet;

#[derive(Resource, Clone, Deref, DerefMut, Debug, Default)]
pub struct LoadedMarkers(pub HashSet<FullMarkerId>);

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(poi::Plugin);
        app.add_plugins(trail::Plugin);
    }
}
