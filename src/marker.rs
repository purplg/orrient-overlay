use std::path::Path;

use bevy::prelude::*;

pub(crate) struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        let content = marker::read(Path::new("markers/tw_aaa.xml")).unwrap();
        println!("content: {:?}", content);
    }
}
