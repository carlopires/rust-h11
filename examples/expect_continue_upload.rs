use h11::{Connection, Data, EndOfMessage, Event, Headers, Response, Role};

fn main() -> Result<(), h11::ProtocolError> {
    let mut conn = Connection::new(Role::Server, None);
    conn.receive_data(
        b"POST /upload HTTP/1.1\r\n\
          Host: example.com\r\n\
          Expect: 100-continue\r\n\
          Content-Length: 5\r\n\r\n",
    )?;

    let Event::Request(request) = conn.next_event()? else {
        panic!("expected request head");
    };
    assert_eq!(request.target, b"/upload");
    assert!(conn.get_they_are_waiting_for_100_continue());

    let continue_response =
        Response::new_informational_http11(100, Headers::default(), "Continue")?;
    let continue_bytes = conn.send(continue_response.into())?.unwrap();
    assert!(continue_bytes.starts_with(b"HTTP/1.1 100 Continue\r\n"));
    assert!(!conn.get_they_are_waiting_for_100_continue());

    assert!(matches!(conn.next_event()?, Event::NeedData()));
    conn.receive_data(b"hello")?;

    let Event::Data(Data { data, .. }) = conn.next_event()? else {
        panic!("expected request body");
    };
    assert_eq!(data, b"hello");
    assert!(matches!(conn.next_event()?, Event::EndOfMessage(_)));

    let final_response =
        Response::new_final_http11(204, Headers::new([("Content-Length", "0")])?, "No Content")?;
    let response_bytes = conn.send(final_response.into())?.unwrap();
    assert!(response_bytes.starts_with(b"HTTP/1.1 204 No Content\r\n"));
    conn.send(EndOfMessage::default().into())?;
    Ok(())
}
