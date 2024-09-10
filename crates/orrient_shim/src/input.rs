use bevy::input::keyboard::NativeKeyCode;
use bevy::input::ButtonState;
use bevy::prelude::*;

use orrient_input::Action;
use orrient_input::ActionEvent;
use orrient_link::SocketMessage;
use rdev::{listen, Event, EventType::*, Key};

use crate::{ChannelRx, ChannelTx};

/// Convert rdev `Key`s into Bevy `KeyCode`s
trait IntoBevyKeyExt {
    fn into_bevy_key(self) -> KeyCode;
}
impl IntoBevyKeyExt for rdev::Key {
    fn into_bevy_key(self) -> KeyCode {
        match self {
            Key::Escape => KeyCode::Escape,
            Key::BackQuote => KeyCode::Backquote,
            Key::Tab => KeyCode::Tab,
            Key::ControlLeft => KeyCode::ControlLeft,
            _ => KeyCode::Unidentified(NativeKeyCode::Unidentified),
        }
    }
}

fn setup(mut commands: Commands) {
    let (tx_input_event, rx_input_event) = crossbeam_channel::unbounded::<rdev::EventType>();
    std::thread::spawn(|| read_input(tx_input_event));
    commands.insert_resource(ChannelRx(rx_input_event));
}

// Thread to read keypresses in the background and send them to the foreground.
fn read_input(tx_input: crossbeam_channel::Sender<rdev::EventType>) {
    if let Err(e) = listen(move |event: Event| {
        if let Err(err) = tx_input.send(event.event_type) {
            println!("err: {:?}", err);
        }
    }) {
        println!("Error when reading input: {:?}", e);
    }
}

/// Read keypresses from the background thread and submit them to ButtonInput<KeyCode>
fn input_system(
    rx_input: Res<ChannelRx<rdev::EventType>>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
) {
    if let Ok(event) = rx_input.try_recv() {
        match event {
            KeyPress(key) => {
                keyboard.press(key.into_bevy_key());
            }
            KeyRelease(key) => {
                keyboard.release(key.into_bevy_key());
            }
            _ => {}
        }
    }
}

/// Read input actions and queue them to be sent over socket
fn action_system(
    mut events: EventReader<ActionEvent>,
    tx: Res<ChannelTx<SocketMessage>>,
    mut modifier_pressed: Local<bool>,
) {
    for event in events.read() {
        match event {
            ActionEvent {
                action: Action::Modifier,
                state: ButtonState::Pressed,
            } => {
                *modifier_pressed = true;
                if let Err(err) = tx.send(SocketMessage::Action(event.clone())) {
                    println!("err: {:?}", err);
                }
            }
            ActionEvent {
                action: Action::Modifier,
                state: ButtonState::Released,
            } => {
                *modifier_pressed = false;
                if let Err(err) = tx.send(SocketMessage::Action(event.clone())) {
                    println!("err: {:?}", err);
                }
            }
            _ => {
                if *modifier_pressed {
                    if let Err(err) = tx.send(SocketMessage::Action(event.clone())) {
                        println!("err: {:?}", err);
                    }
                }
            }
        }
    }
}

pub(crate) struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy::input::InputPlugin);
        app.add_plugins(orrient_input::Plugin);
        app.add_systems(Startup, setup);
        app.add_systems(Update, input_system);
        app.add_systems(Update, action_system.run_if(on_event::<ActionEvent>()));
    }
}
