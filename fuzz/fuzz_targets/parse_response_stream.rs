#![no_main]

use h11::{Connection, Event, Role};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut conn = Connection::new(Role::Client, None);
    let _ = conn.receive_data(data);

    for _ in 0..32 {
        match conn.next_event() {
            Ok(Event::NeedData()) | Ok(Event::Paused()) | Err(_) => break,
            Ok(_) => {}
        }
    }
});
