# RFC 9112 Compliance Notes

This is a working checklist for HTTP/1.1 message syntax, parsing, framing, and
connection management. It is intentionally scoped to h11's Sans-I/O protocol
core; transport, TLS, caching, authentication, routing, and application
semantics belong outside this crate or in RFC 9110.

## Status Key

- `covered`: implemented with direct test coverage.
- `partial`: implemented in part or missing a specific RFC nuance.
- `out-of-scope`: intentionally outside a Sans-I/O HTTP/1.1 core.
- `needs-tests`: behavior exists but needs more explicit coverage.

## Matrix

| RFC 9112 section | Requirement area | Status | Test coverage | Notes |
| --- | --- | --- | --- | --- |
| 2.1 | Message format: start line, field section, blank line, optional content | covered | `_connection::tests::test_connection_basics_and_content_length`, `_connection::tests::test_chunked`, `tests/python_h11_fixtures.rs` | Parser and writer expose this as event sequences. |
| 2.2 | Message parsing robustness | partial | `_connection::tests::test_errors`, `_connection::tests::test_early_detection_of_invalid_request`, `_connection::tests::test_early_detection_of_invalid_response`, `tests/python_h11_fixtures.rs` | Early rejection exists, and Python h11 fixtures cover minimized malformed header/start-line/framing cases. Fuzz regression coverage should continue to expand. |
| 2.3 | HTTP version parsing and handling | covered | `tests/differential_httparse.rs`, `_events::tests::test_request_new_rejects_invalid_http_version`, `_events::tests::test_response_new_rejects_invalid_input` | Accepts HTTP/1.x wire versions and tracks peer version for behavior. |
| 3 | Request line syntax | covered | `tests/differential_httparse.rs`, `_connection::tests::test_empty_request`, `_connection::tests::test_early_detection_of_invalid_request` | Validates method token and target bytes. |
| 3.2 | Request target forms | partial | `_events::tests::test_request_new_accepts_borrowed_inputs_and_http11_default` | Character validation exists; origin-form, absolute-form, authority-form, and asterisk-form are not classified. |
| 4 | Status line syntax | covered | `_events::tests::test_response_new_rejects_invalid_input`, `_events::tests::test_response_range_checked_constructors`, `tests/differential_httparse.rs` | Status code, reason phrase, version, and informational/final ranges are validated by constructors. |
| 5 | Field syntax and parsing | covered | `_headers::tests::test_normalize_and_validate`, `_headers::tests::test_headers_new_accepts_borrowed_inputs`, `tests/python_h11_fixtures.rs` | Field names/values are validated, normalized for lookup, and raw names are preserved for writing. Python h11 fixtures cover malformed header lines and invalid continuation placement. |
| 5.2 | Obsolete line folding | covered | `_readers::tests::test_obsolete_line_fold_bytes`, `tests/python_h11_fixtures.rs` | Received obs-fold lines are unfolded before header decoding and compared against Python h11 fixtures. |
| 6 | Message body presence and framing | covered | `_connection::tests::test_body_framing`, `_connection::tests::test_head_framing_headers`, `_connection::tests::test_special_exceptions_for_lost_connection_in_message_body` | Handles request bodies, response bodies, HEAD, 204/304, CONNECT success, and EOF cases. |
| 6.1 | Transfer-Encoding | partial | `_headers::tests::test_normalize_and_validate`, `_connection::tests::test_chunked`, `tests/python_h11_fixtures.rs` | Only `chunked` is supported; unsupported transfer codings are rejected with 501 and covered by Python h11 fixtures. |
| 6.2 | Content-Length | covered | `_headers::tests::test_normalize_and_validate`, `_connection::tests::test_connection_basics_and_content_length`, `tests/python_h11_fixtures.rs` | Rejects invalid values and conflicting duplicates; duplicate matching values are accepted and normalized. |
| 7.1 | Chunked transfer coding | covered | `_connection::tests::test_chunked`, `_connection::tests::test_chunk_boundaries`, `_readers::tests::test_chunked_reader`, `_writers::tests::test_chunked_writer`, `tests/python_h11_fixtures.rs` | Parses and serializes chunked bodies. |
| 7.1.1 | Chunk extensions | partial | `_readers::tests::test_chunked_reader` | Chunk extensions are parsed and ignored; no semantic extension handling is exposed. |
| 7.1.2 | Trailer fields | partial | `_connection::tests::test_chunked`, `tests/python_h11_fixtures.rs` | Trailers parse and serialize, but field-specific trailer restrictions are not enforced. |
| 8 | Incomplete messages | covered | `_connection::tests::test_special_exceptions_for_lost_connection_in_message_body`, `_connection::tests::test_connection_drop` | Incomplete fixed-length and chunked bodies produce remote protocol errors. |
| 9.1 | Connection management foundation | covered | `_connection::tests::test_close_simple`, `_connection::tests::test_close_different_states`, `_connection::tests::test_automagic_connection_close_handling` | Tracks local/remote close state and EOF handling. |
| 9.3 | Persistence | covered | `_connection::tests::test_keep_alive`, `_connection::tests::test_reuse_simple`, `_state::tests::test_connection_state_reuse` | Supports keep-alive cycles and close-triggered must-close states. |
| 9.3.2 | Pipelining | covered | `_connection::tests::test_pipelining`, `_connection::tests::test_pipelined_close` | Preserves trailing pipelined bytes and pauses until the next cycle. |
| 9.7 | TLS connection initiation | out-of-scope | none | TLS belongs to the caller's transport layer. |
| 9.8 | TLS connection closure | out-of-scope | none | TLS shutdown belongs to the caller's transport layer. |
| 11 | Security considerations | partial | `_connection::tests::test_max_incomplete_event_size_countermeasure`, `_headers::tests::test_normalize_and_validate`, fuzz targets and seed corpora under `fuzz/` | Buffer caps, framing-conflict rejection, and fuzz seeds exist. Regular fuzz runs and regression promotion are still needed. |

## Not Covered

- HTTP/2 and HTTP/3 negotiation or framing.
- URI reconstruction, authority validation, proxy routing, and DNS behavior.
- Header-specific semantics beyond framing-critical fields.
- Content codings such as gzip or deflate.
- Cache semantics, conditional requests, range requests, authentication, and
  cookies.
- Production transport concerns such as timeouts, socket shutdown ordering,
  retry policy, connection pools, and backpressure.

## Hardening Plan

- Keep differential tests against `httparse` for common start-line acceptance
  and rejection behavior.
- Run cargo-fuzz targets before releases and after parser/state-machine changes:
  `cargo fuzz run parse_request_stream` and
  `cargo fuzz run parse_response_stream`.
- Expand the fuzz corpora as new request-smuggling, response-splitting, chunk,
  obs-fold, and EOF cases are found.
- Promote minimized fuzz crashes into regression tests.
- Expand this checklist as behavior is reviewed against RFC 9112 section by
  section.
