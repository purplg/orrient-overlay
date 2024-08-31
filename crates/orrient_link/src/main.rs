mod input;
mod link;
mod net;

use std::ops::Deref;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

fn main() {
    App::new()
        .add_plugins(bevy_app::ScheduleRunnerPlugin::default())
        .add_plugins(input::Plugin)
        .add_plugins(net::Plugin)
        .add_plugins(link::Plugin)
        .run();
}

/// Channel to receive rdev key events
#[derive(Resource)]
struct ChannelRx<T>(crossbeam_channel::Receiver<T>);
impl<T> Deref for ChannelRx<T> {
    type Target = crossbeam_channel::Receiver<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Resource)]
struct ChannelTx<T>(crossbeam_channel::Sender<T>);
impl<T> Deref for ChannelTx<T> {
    type Target = crossbeam_channel::Sender<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
