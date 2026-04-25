# Cookbook

These examples show protocol-control patterns that are easy to get subtly
wrong when wiring `h11` into a transport loop. The source files under
`examples/` are compiled by Cargo, so they stay aligned with the public API.

## Pipelined Requests

Use `Event::Paused` as the boundary between a completed request and buffered
bytes for the next request. Finish the current response, call
`start_next_cycle`, and then continue reading.

```bash
cargo run --example pipelined_server
```

## `100-continue`

Servers can inspect `get_they_are_waiting_for_100_continue` after receiving a
request head. Sending an informational `100 Continue` response clears that
state and allows the client to send the body.

```bash
cargo run --example expect_continue_upload
```

## Upgrade Handoff

After accepting an Upgrade request with `101 Switching Protocols`, HTTP parsing
pauses. Use `get_trailing_data` to recover bytes that were already read beyond
the HTTP message and pass them to the upgraded protocol.

```bash
cargo run --example upgrade_handoff
```
