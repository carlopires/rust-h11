# Fuzz Corpus Seeds

These files seed `cargo fuzz` with HTTP/1.1 edge cases that should stay
interesting for parser and state-machine hardening.

Run examples:

```bash
cargo fuzz run parse_request_stream
cargo fuzz run parse_response_stream
```

The seed names describe the behavior they exercise. Request seeds are most
useful for `parse_request_stream`; response seeds are most useful for
`parse_response_stream`. The fuzz targets still accept arbitrary bytes, so it is
fine if a seed is rejected with a protocol error.
