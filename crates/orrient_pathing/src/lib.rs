mod events;
mod marker;
mod parser;

use bevy::prelude::*;

pub mod prelude {
    pub use crate::events::MarkerEvent;
    pub use crate::events::ReloadMarkersEvent;
    pub use crate::marker::poi::PoiMarker;
    pub use crate::marker::trail::create_trail_mesh;
    pub use crate::marker::trail::TrailMaterial;
    pub use crate::marker::trail::TrailMesh;
    pub use crate::marker::EnabledMarkers;
    pub use crate::parser::model::Poi;
    pub use crate::parser::pack::Behavior;
    pub use crate::parser::pack::FullMarkerId;
    pub use crate::parser::pack::Marker;
    pub use crate::parser::pack::MarkerId;
    pub use crate::parser::pack::MarkerKind;
    pub use crate::parser::pack::MarkerPack;
    pub use crate::parser::MarkerPacks;
    pub use crate::parser::PackId;
}

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(marker::Plugin);
        app.add_plugins(parser::Plugin);
        app.add_plugins(events::Plugin);
    }
}
