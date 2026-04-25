# User Guide

`h11` is a Sans-I/O HTTP/1.1 protocol core. It parses bytes into events and
serializes events into bytes, but it never opens sockets, waits on timers, or
chooses an async runtime.

The caller owns the transport loop:

1. Read bytes from a socket, TLS stream, pipe, or test fixture.
2. Feed those bytes to `Connection::receive_data`.
3. Pull inbound protocol events with `Connection::next_event`.
4. Build outbound events and pass them to `Connection::send`.
5. Write returned bytes to the transport.

## Basic Client

A client creates a `Connection` with `Role::Client`, sends a request head,
optionally sends body data, then reads response events.

```rust
use h11::{Connection, EndOfMessage, Event, Headers, Request, Role};

fn build_get_request() -> Result<Vec<Vec<u8>>, h11::ProtocolError> {
    let mut conn = Connection::new(Role::Client, None);
    let request = Request::new_http11(
        "GET",
        Headers::new([("Host", "example.com"), ("User-Agent", "example")])?,
        "/",
    )?;

    let mut out = Vec::new();
    out.push(conn.send(request.into())?.unwrap_or_default());
    out.push(conn.send(EndOfMessage::default().into())?.unwrap_or_default());
    Ok(out)
}

fn handle_response(conn: &mut Connection, bytes: &[u8]) -> Result<(), h11::ProtocolError> {
    conn.receive_data(bytes)?;
    loop {
        match conn.next_event()? {
            Event::NormalResponse(response) => {
                println!("status={}", response.status_code);
            }
            Event::Data(data) => {
                println!("body chunk: {} bytes", data.data.len());
            }
            Event::EndOfMessage(_) | Event::NeedData() | Event::Paused() => break,
            Event::ConnectionClosed(_) => break,
            event => println!("unexpected event: {event:?}"),
        }
    }
    Ok(())
}
```

`Connection::send` returns `Ok(Some(bytes))` when an event produces wire bytes
and `Ok(None)` for `ConnectionClosed`.

## Basic Server

A server creates a `Connection` with `Role::Server`, feeds request bytes, then
responds after it has consumed a request.

```rust
use h11::{Connection, EndOfMessage, Event, Headers, Response, Role};

fn handle_request(bytes: &[u8]) -> Result<Vec<Vec<u8>>, h11::ProtocolError> {
    let mut conn = Connection::new(Role::Server, None);
    conn.receive_data(bytes)?;

    loop {
        match conn.next_event()? {
            Event::Request(request) => {
                println!(
                    "{} {}",
                    String::from_utf8_lossy(&request.method),
                    String::from_utf8_lossy(&request.target)
                );
            }
            Event::EndOfMessage(_) => break,
            Event::NeedData() => return Ok(Vec::new()),
            event => println!("unexpected event: {event:?}"),
        }
    }

    let response = Response::new_final_http11(
        200,
        Headers::new([("Content-Length", "0")])?,
        "OK",
    )?;

    let mut out = Vec::new();
    out.push(conn.send(response.into())?.unwrap_or_default());
    out.push(conn.send(EndOfMessage::default().into())?.unwrap_or_default());
    Ok(out)
}
```

If `next_event` returns `NeedData`, read more bytes from the transport and call
`receive_data` again. Passing an empty byte slice marks EOF.

## Bodies and Trailers

Body bytes are represented by `Event::Data`. The end of each request or
response is represented by `Event::EndOfMessage`.

For fixed-length bodies, include `Content-Length` and send exactly that many
bytes:

```rust
use h11::{Connection, Data, EndOfMessage, Headers, Request, Role};

fn post_body() -> Result<Vec<Vec<u8>>, h11::ProtocolError> {
    let mut conn = Connection::new(Role::Client, None);
    let request = Request::new_http11(
        "POST",
        Headers::new([("Host", "example.com"), ("Content-Length", "5")])?,
        "/upload",
    )?;

    Ok(vec![
        conn.send(request.into())?.unwrap_or_default(),
        conn.send(
            Data {
                data: b"hello".to_vec(),
                ..Default::default()
            }
            .into(),
        )?
        .unwrap_or_default(),
        conn.send(EndOfMessage::default().into())?.unwrap_or_default(),
    ])
}
```

For chunked responses, omit `Content-Length` on an HTTP/1.1 response after a
request has been consumed. The connection will add
`Transfer-Encoding: chunked` when appropriate. Trailer headers can be carried
on `EndOfMessage`.

```rust
use h11::{Connection, Data, EndOfMessage, Headers, Response};

fn chunked_response(conn: &mut Connection) -> Result<Vec<Vec<u8>>, h11::ProtocolError> {
    let response = Response::new_final_http11(200, Headers::default(), "OK")?;

    Ok(vec![
        conn.send(response.into())?.unwrap_or_default(),
        conn.send(
            Data {
                data: b"hello".to_vec(),
                ..Default::default()
            }
            .into(),
        )?
        .unwrap_or_default(),
        conn.send(
            EndOfMessage {
                headers: Headers::new([("Etag", "abc")])?,
            }
            .into(),
        )?
        .unwrap_or_default(),
    ])
}
```

## Keep-Alive and Pipelining

After both sides reach `State::Done`, call `start_next_cycle` to reuse the same
connection for the next request/response pair.

```rust
use h11::{Connection, State};

fn maybe_reuse(conn: &mut Connection) -> Result<(), h11::ProtocolError> {
    if conn.get_our_state() == State::Done && conn.get_their_state() == State::Done {
        conn.start_next_cycle()?;
    }
    Ok(())
}
```

If the peer pipelines a second request behind the first, `next_event` returns
`Event::Paused` after the first message completes. Finish the current response,
call `start_next_cycle`, then continue reading events.

Use `get_trailing_data` if the protocol switches away from HTTP/1.1 or you need
the already-buffered bytes after a pause.

## `100-continue`

Servers can check whether the client is waiting for `100 Continue`.

```rust
use h11::{Connection, Headers, Response};

fn maybe_send_continue(conn: &mut Connection) -> Result<Option<Vec<u8>>, h11::ProtocolError> {
    if conn.get_they_are_waiting_for_100_continue() {
        let response = Response::new_informational_http11(
            100,
            Headers::default(),
            "Continue",
        )?;
        return conn.send(response.into());
    }
    Ok(None)
}
```

The waiting flag is cleared when an informational or final response is sent, or
when client body data arrives.

## CONNECT and Upgrade

Protocol switching is represented by state transitions. A client proposes a
switch with a `CONNECT` request or an `Upgrade` header. If the server accepts
the switch, both sides move to `State::SwitchedProtocol`.

For CONNECT, a successful `2xx` response accepts the tunnel. For Upgrade, a
`101 Switching Protocols` informational response accepts the switch.

Once `next_event` returns `Paused` in a switched state, HTTP parsing is done.
Use `get_trailing_data` to recover already-buffered bytes and then hand the
transport back to the upgraded protocol.

## Error Handling

Most fallible APIs return `ProtocolError`.

- `LocalProtocolError` means local code attempted an invalid event, invalid
  sequence, or invalid locally-built message.
- `RemoteProtocolError` means peer bytes violated the protocol.

After an error, the corresponding side of the state machine is moved to
`State::Error`. If writing returned bytes to the transport fails, call
`send_failed` so the connection state reflects that failure.

```rust
use h11::{Connection, ProtocolError, Role};

fn receive(conn: &mut Connection, bytes: &[u8]) {
    if let Err(error) = conn.receive_data(bytes).and_then(|_| conn.next_event().map(|_| ())) {
        match error {
            ProtocolError::LocalProtocolError(error) => eprintln!("local error: {error}"),
            ProtocolError::RemoteProtocolError(error) => eprintln!("remote error: {error}"),
        }
    }
}

let mut conn = Connection::new(Role::Server, None);
receive(&mut conn, b"not http\r\n\r\n");
```

## Manual Event Construction

Prefer fallible constructors such as `Headers::new`, `Request::new_http11`,
`Response::new_informational_http11`, and `Response::new_final_http11`.

Struct fields are public for compatibility, but direct struct literals bypass
constructor validation. If you build events manually from untrusted inputs, call
`validate` before sending request or response heads.
