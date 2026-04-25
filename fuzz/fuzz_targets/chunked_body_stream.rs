#![no_main]

use h11::{Connection, Event, Role};
use libfuzzer_sys::fuzz_target;

fn drain(conn: &mut Connection) {
    for _ in 0..64 {
        match conn.next_event() {
            Ok(Event::NeedData()) | Ok(Event::Paused()) | Err(_) => break,
            Ok(_) => {}
        }
    }
}

fuzz_target!(|data: &[u8]| {
    let (role, head, body) = if data.first().copied().unwrap_or(0) & 1 == 0 {
        (
            Role::Server,
            b"POST /upload HTTP/1.1\r\nHost: example.com\r\nTransfer-Encoding: chunked\r\n\r\n"
                .as_slice(),
            data,
        )
    } else {
        (
            Role::Client,
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n".as_slice(),
            data,
        )
    };

    let mut conn = Connection::new(role, None);
    let _ = conn.receive_data(head);
    drain(&mut conn);
    let _ = conn.receive_data(body);
    drain(&mut conn);
});
