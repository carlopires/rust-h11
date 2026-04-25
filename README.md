# h11

Inspired by [python-h11](https://github.com/python-hyper/h11).

It's a "bring-your-own-I/O" library; h11 contains no IO code whatsoever. This means you can hook h11 up to your favorite network API, and that could be anything you want: synchronous, threaded, asynchronous, or your own implementation of [RFC 6214](https://www.rfc-editor.org/rfc/rfc6214) -- h11 won't judge you.

## Install

```bash
cargo add h11
```

## Basic Usage

`h11` manages HTTP/1.1 protocol state and serialization, but leaves socket or
async I/O to the caller.

```rust
use h11::{Connection, EndOfMessage, Event, Headers, Request, Response, Role};

fn main() -> Result<(), h11::ProtocolError> {
    let mut client = Connection::new(Role::Client, None);
    let request = Request::new(
        b"GET".to_vec(),
        Headers::new(vec![(b"Host".to_vec(), b"example.com".to_vec())])?,
        b"/".to_vec(),
        b"1.1".to_vec(),
    )?;

    let bytes_to_send = client.send(request.into())?.unwrap();
    assert_eq!(
        bytes_to_send,
        b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n".to_vec()
    );

    let mut server = Connection::new(Role::Server, None);
    server.receive_data(&bytes_to_send)?;

    match server.next_event()? {
        Event::Request(request) => assert_eq!(request.target, b"/".to_vec()),
        event => panic!("unexpected event: {event:?}"),
    }

    let response = Response::new(
        200,
        Headers::new(vec![(b"Content-Length".to_vec(), b"0".to_vec())])?,
        b"OK".to_vec(),
        b"1.1".to_vec(),
    )?;

    let response_bytes = server.send(response.into())?.unwrap();
    assert_eq!(
        response_bytes,
        b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n".to_vec()
    );

    server.send(EndOfMessage::default().into())?;
    Ok(())
}
```
