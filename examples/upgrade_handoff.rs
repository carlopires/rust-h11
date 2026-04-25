use h11::{Connection, Event, Headers, Response, Role, State};

fn main() -> Result<(), h11::ProtocolError> {
    let mut conn = Connection::new(Role::Server, None);
    conn.receive_data(
        b"GET /chat HTTP/1.1\r\n\
          Host: example.com\r\n\
          Connection: upgrade\r\n\
          Upgrade: websocket\r\n\r\n\
          raw-websocket-bytes",
    )?;

    let Event::Request(request) = conn.next_event()? else {
        panic!("expected upgrade request");
    };
    assert_eq!(request.target, b"/chat");
    assert!(matches!(conn.next_event()?, Event::EndOfMessage(_)));

    let switching = Response::new_informational_http11(
        101,
        Headers::new([("Connection", "upgrade"), ("Upgrade", "websocket")])?,
        "Switching Protocols",
    )?;
    let response_bytes = conn.send(switching.into())?.unwrap();
    assert!(response_bytes.starts_with(b"HTTP/1.1 101 Switching Protocols\r\n"));
    assert_eq!(conn.get_our_state(), State::SwitchedProtocol);

    assert!(matches!(conn.next_event()?, Event::Paused()));
    assert_eq!(
        conn.get_trailing_data(),
        (b"raw-websocket-bytes".to_vec(), false)
    );
    Ok(())
}
