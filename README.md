# h11

Inspired by [python-h11](https://github.com/python-hyper/h11).

It's a "bring-your-own-I/O" library; h11 contains no IO code whatsoever. This means you can hook h11 up to your favorite network API, and that could be anything you want: synchronous, threaded, asynchronous, or your own implementation of [RFC 6214](https://www.rfc-editor.org/rfc/rfc6214) -- h11 won't judge you.

See [the user guide](docs/user-guide.md) for client/server loops, bodies,
keep-alive, `100-continue`, protocol switching, and error handling.

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
    let request =
        Request::new_http11("GET", Headers::new([("Host", "example.com")])?, "/")?;

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

    let response = Response::new_http11(
        200,
        Headers::new([("Content-Length", "0")])?,
        "OK",
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

## Development

Run parser and serializer benchmarks before changing parsing internals:

```bash
cargo bench --bench parser
```

See [performance baselines](docs/performance.md) for comparison guidance.
