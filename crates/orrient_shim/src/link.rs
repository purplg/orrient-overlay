use crate::{ChannelRx, ChannelTx};
use orrient_link::{MumbleLinkDataDef, SocketMessage};

use bevy::prelude::*;

use std::thread::sleep;
use std::time::Duration;

use mumblelink_reader::mumble_link::MumbleLinkReader;
use mumblelink_reader::mumble_link_handler::MumbleLinkHandler;

fn link(tx: crossbeam_channel::Sender<SocketMessage>) {
    let handler = MumbleLinkHandler::new().unwrap();

    info!("Connecting to MumbleLink...");

    loop {
        match handler.read() {
            Ok(data) => {
                if data.ui_tick > 0 {
                    // If we have any ui_tick's, then GW2 should be
                    // running and we can continue to the next loop to
                    // actually parse and send the data.
                    break;
                } else {
                    // Otherwise, we wait a second to try again.
                    sleep(Duration::from_secs(1));
                }
            }
            Err(error) => {
                error!("Could not read data... weird: {:?}", error);
                sleep(Duration::from_secs(1));
            }
        }
    }

    info!("Connected!");
    let mut last_ui_tick: i64 = 0;

    // The amount of millis to sleep until the next time we try to
    // read mumblelink.
    let mut tick_rate = 0;

    // Keeps track of the number of times the `ui_tick` field has no
    // been updated. If this exceeds 3, then we'll slow down the send
    // rate a bit. This is to mitigate stuttering by only decreasing
    // the send rate if we repeated read mumblelink too fast.
    let mut fast_count = 0;

    loop {
        sleep(Duration::from_millis(tick_rate));

        let data = match handler.read() {
            Ok(data) => data,
            Err(error) => {
                error!("Could not read data... weird: {:?}", error);
                continue;
            }
        };

        let def = match MumbleLinkDataDef::from_data(data) {
            Ok(def) => def,
            Err(error) => {
                error!("Error deserializing data: {:?}", error);
                continue;
            }
        };

        if def.ui_tick > last_ui_tick {
            // We got a new frame, so store this ui_tick as the last
            // one.
            last_ui_tick = def.ui_tick;
            if let Err(e) = tx.send(SocketMessage::MumbleLinkData(Box::new(def))) {
                error!("{:?}", e);
            };
            // ... and decrease the tickrate.
            if tick_rate > 0 {
                tick_rate /= 2;
            }
            // ... reset the error count
            fast_count = 0;
        } else {
            fast_count += 1;
            if fast_count > 3 {
                tick_rate += 1;
            }
        }
    }
}

fn setup(mut commands: Commands) {
    let (tx_link, rx_link) = crossbeam_channel::unbounded::<SocketMessage>();
    commands.insert_resource(ChannelTx(tx_link.clone()));
    commands.insert_resource(ChannelRx(rx_link));
    std::thread::spawn(|| link(tx_link));
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}
