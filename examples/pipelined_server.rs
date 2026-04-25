use h11::{Connection, EndOfMessage, Event, Headers, Response, Role};

fn read_request_target(conn: &mut Connection) -> Result<Vec<u8>, h11::ProtocolError> {
    let Event::Request(request) = conn.next_event()? else {
        panic!("expected request");
    };
    assert!(matches!(conn.next_event()?, Event::EndOfMessage(_)));
    Ok(request.target)
}

fn send_empty_response(conn: &mut Connection) -> Result<Vec<u8>, h11::ProtocolError> {
    let response =
        Response::new_final_http11(204, Headers::new([("Content-Length", "0")])?, "No Content")?;
    let mut out = conn.send(response.into())?.unwrap();
    out.extend(conn.send(EndOfMessage::default().into())?.unwrap());
    Ok(out)
}

fn main() -> Result<(), h11::ProtocolError> {
    let mut conn = Connection::new(Role::Server, None);
    conn.receive_data(
        b"GET /one HTTP/1.1\r\nHost: example.com\r\n\r\n\
          GET /two HTTP/1.1\r\nHost: example.com\r\n\r\n",
    )?;

    assert_eq!(read_request_target(&mut conn)?, b"/one");
    assert!(matches!(conn.next_event()?, Event::Paused()));

    let mut wire_bytes = send_empty_response(&mut conn)?;
    conn.start_next_cycle()?;

    assert_eq!(read_request_target(&mut conn)?, b"/two");
    wire_bytes.extend(send_empty_response(&mut conn)?);

    assert!(wire_bytes.starts_with(b"HTTP/1.1 204 No Content\r\n"));
    assert_eq!(conn.get_trailing_data(), (Vec::new(), false));
    Ok(())
}
