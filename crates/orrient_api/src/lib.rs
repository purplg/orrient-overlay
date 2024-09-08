mod blish_hud;
mod gw2;

use bevy::prelude::*;

pub mod prelude {
    pub use crate::blish_hud::DownloadablePacks;
    pub use crate::blish_hud::BHAPIEvent;
    pub use crate::blish_hud::RepoPack;
    pub use crate::blish_hud::RepoPackId;
    pub use crate::gw2::Endpoint as GW2Endpoint;
    pub use crate::gw2::RequestComplete as GW2RequestComplete;
    pub use crate::gw2::RequestQueue as GW2RequestQueue;
    pub use crate::gw2::Response as GW2Response;
}

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(gw2::Plugin);
        app.add_plugins(blish_hud::Plugin);
    }
}
