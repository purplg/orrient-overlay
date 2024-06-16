use std::cell::Cell;
use std::thread::sleep;
use std::time::Duration;
use std::{net::UdpSocket, rc::Rc};

use mumblelink::{MumbleLinkDataDef, MumbleLinkMessage};
use mumblelink_reader::mumble_link::MumbleLinkReader;
use mumblelink_reader::mumble_link_handler::MumbleLinkHandler;
use rdev::{listen, Event, EventType::*, Key};
use serde::{Deserialize, Serialize};

fn main() {
    let (tx, rx) = crossbeam_channel::unbounded::<MumbleLinkMessage>();

    let socket = UdpSocket::bind("127.0.0.1:5000").unwrap();

    let tx_input = tx.clone();
    std::thread::spawn(|| input(tx_input));
    std::thread::spawn(|| link(tx));

    while let Ok(message) = rx.recv() {
        let buf = bincode::serialize(&message).unwrap();
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
                            println!("Error sending toggle message: {:?}", e);
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
    loop {
        tx.send(MumbleLinkMessage::MumbleLinkData(
            handler.read().unwrap().into(),
        ));
        sleep(Duration::from_millis(32));
    }
}
