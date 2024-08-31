pub mod events;
pub mod marker;
pub mod parser;

use bevy::prelude::*;

pub mod prelude {
    pub use crate::parser::pack::Behavior;
    pub use crate::parser::pack::FullMarkerId;
    pub use crate::parser::pack::Marker;
    pub use crate::parser::pack::MarkerId;
    pub use crate::parser::pack::MarkerKind;
    pub use crate::parser::pack::MarkerPack;
    pub use crate::parser::MarkerPacks;
    pub use crate::parser::PackId;
    pub use crate::events::LoadPoiEvent;
    pub use crate::events::LoadTrailEvent;
    pub use crate::events::MarkerEvent;
}

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(marker::Plugin);
        app.add_plugins(parser::Plugin);
        app.add_plugins(events::Plugin);
    }
}
