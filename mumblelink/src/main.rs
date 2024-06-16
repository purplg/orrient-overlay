use std::net::UdpSocket;
use std::thread::sleep;
use std::time::Duration;

use mumblelink::MumbleLinkDataDef;
use mumblelink_reader::mumble_link::MumbleLinkReader;
use mumblelink_reader::mumble_link_handler::MumbleLinkHandler;

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:5000").unwrap();
    let handler = MumbleLinkHandler::new().unwrap();
    loop {
        let a: MumbleLinkDataDef = handler.read().unwrap().into();
        let buf = bincode::serialize(&a).unwrap();
        let _ = socket.send_to(buf.as_slice(), "127.0.0.1:5001");
        sleep(Duration::from_millis(32))
    }
}
