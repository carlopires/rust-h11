#![no_main]

use h11::{Connection, EndOfMessage, Event, Headers, Request, Response, Role, State};
use libfuzzer_sys::fuzz_target;

fn token_byte(byte: u8) -> u8 {
    b'a' + (byte % 26)
}

fn target_from_bytes(data: &[u8]) -> Vec<u8> {
    let mut target = Vec::with_capacity(data.len().min(16) + 1);
    target.push(b'/');
    for byte in data.iter().take(16) {
        target.push(token_byte(*byte));
    }
    target
}

fn drain(conn: &mut Connection) -> Result<Vec<Event>, h11::ProtocolError> {
    let mut events = Vec::new();
    for _ in 0..16 {
        match conn.next_event()? {
            Event::NeedData() | Event::Paused() => break,
            event => events.push(event),
        }
    }
    Ok(events)
}

fn transfer(
    from: &mut Connection,
    to: &mut Connection,
    event: Event,
) -> Result<(), h11::ProtocolError> {
    if let Some(bytes) = from.send(event)? {
        to.receive_data(&bytes)?;
    }
    Ok(())
}

fuzz_target!(|data: &[u8]| {
    let mut client = Connection::new(Role::Client, None);
    let mut server = Connection::new(Role::Server, None);

    for chunk in data.chunks(4).take(8) {
        let target = target_from_bytes(chunk);
        let request = match Request::new_http11(
            "GET",
            Headers::new([("Host", "example.com")]).unwrap(),
            target,
        ) {
            Ok(request) => request,
            Err(_) => break,
        };
        if transfer(&mut client, &mut server, request.into()).is_err() {
            break;
        }
        if transfer(&mut client, &mut server, EndOfMessage::default().into()).is_err() {
            break;
        }
        if drain(&mut server).is_err() {
            break;
        }

        let status = if chunk.first().copied().unwrap_or(0) & 1 == 0 {
            204
        } else {
            200
        };
        let response = match Response::new_final_http11(
            status,
            Headers::new([("Content-Length", "0")]).unwrap(),
            "OK",
        ) {
            Ok(response) => response,
            Err(_) => break,
        };
        if transfer(&mut server, &mut client, response.into()).is_err() {
            break;
        }
        if transfer(&mut server, &mut client, EndOfMessage::default().into()).is_err() {
            break;
        }
        if drain(&mut client).is_err() {
            break;
        }

        if client.get_our_state() == State::Done
            && client.get_their_state() == State::Done
            && server.get_our_state() == State::Done
            && server.get_their_state() == State::Done
        {
            let _ = client.start_next_cycle();
            let _ = server.start_next_cycle();
        } else {
            break;
        }
    }
});
