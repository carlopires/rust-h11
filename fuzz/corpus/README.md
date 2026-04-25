# Fuzz Corpus Seeds

These files seed `cargo fuzz` with HTTP/1.1 edge cases that should stay
interesting for parser and state-machine hardening.

Run examples:

```bash
cargo fuzz run parse_request_stream
cargo fuzz run parse_response_stream
cargo fuzz run chunked_body_stream
cargo fuzz run state_machine_roundtrip
```

The seed names describe the behavior they exercise. Request seeds are most
useful for `parse_request_stream`; response seeds are most useful for
`parse_response_stream`; chunked-body seeds are most useful for
`chunked_body_stream`; state-machine seeds are most useful for
`state_machine_roundtrip`. The fuzz targets still accept arbitrary bytes, so it
is fine if a seed is rejected with a protocol error.
