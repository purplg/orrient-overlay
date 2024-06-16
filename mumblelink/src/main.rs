use std::thread::sleep;
use std::time::Duration;

use mumblelink_reader::mumble_link::MumbleLinkReader;
use mumblelink_reader::mumble_link_handler::MumbleLinkHandler;

fn main() {
    let handler = MumbleLinkHandler::new().unwrap();
    loop {
        let a = handler.read().unwrap();
        println!("{:?}", a);
        sleep(Duration::from_millis(50))
    }
}
