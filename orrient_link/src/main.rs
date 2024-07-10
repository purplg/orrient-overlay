use std::net::UdpSocket;
use std::thread::sleep;
use std::time::Duration;

use bincode::Options as _;
use mumblelink_reader::mumble_link::MumbleLinkReader;
use mumblelink_reader::mumble_link_handler::MumbleLinkHandler;
use orrient_link::{MumbleLinkDataDef, MumbleLinkMessage};
use rdev::{listen, Event, EventType::*, Key};

fn main() {
    let (tx, rx) = crossbeam_channel::unbounded::<MumbleLinkMessage>();

    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();

    let tx_input = tx.clone();
    std::thread::spawn(|| input(tx_input));
    std::thread::spawn(|| link(tx));

    while let Ok(message) = rx.recv() {
        let buf = bincode::DefaultOptions::new()
            .allow_trailing_bytes()
            .serialize(&message)
            .unwrap();
        let _ = socket.send_to(buf.as_slice(), "127.0.0.1:5001");
    }
}

fn input(tx: crossbeam_channel::Sender<MumbleLinkMessage>) {
    let mut mod_held = false;
    if let Err(e) = listen(move |event: Event| {
        if mod_held {
            match event.event_type {
                KeyPress(key) => match key {
                    Key::Comma => {
                        if let Err(e) = tx.send(MumbleLinkMessage::Toggle) {
                            println!("Error sending Toggle message: {:?}", e);
                        }
                    }
                    Key::Dot => {
                        if let Err(e) = tx.send(MumbleLinkMessage::Save) {
                            println!("Error sending Save message: {:?}", e);
                        }
                    }
                    _ => {}
                },
                KeyRelease(key) => match key {
                    Key::ControlLeft | Key::ControlRight => mod_held = false,
                    _ => {}
                },
                _ => {}
            }
        } else {
            match event.event_type {
                KeyPress(key) => match key {
                    Key::ControlLeft | Key::ControlRight => mod_held = true,
                    _ => {}
                },
                _ => {}
            }
        }
    }) {
        println!("e: {:?}", e);
    }
}

fn link(tx: crossbeam_channel::Sender<MumbleLinkMessage>) {
    let handler = MumbleLinkHandler::new().unwrap();

    println!("Waiting for MumbleLink...");

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
                println!("Could not read data... weird: {:?}", error);
                sleep(Duration::from_secs(1));
            }
        }
    }

    println!("Connected!");
    let mut ui_tick: i64 = 0;
    let mut tick_rate = 0;
    loop {
        sleep(Duration::from_millis(tick_rate));
        let data = match handler.read() {
            Ok(data) => data,
            Err(error) => {
                println!("Could not read data... weird: {:?}", error);
                continue;
            }
        };

        let def = match MumbleLinkDataDef::from_data(data.clone()) {
            Ok(def) => def,
            Err(error) => {
                println!("Error deserializing data: {:?}\n{:?}", error, data);
                continue;
            }
        };

        if def.ui_tick > ui_tick {
            if tick_rate > 0 {
                tick_rate -= 1;
            }
            ui_tick = def.ui_tick;
            if let Err(e) = tx.send(MumbleLinkMessage::MumbleLinkData(Box::new(def))) {
                println!("error: {:?}", e);
            };
        } else {
            tick_rate += 1;
        }
    }
}
