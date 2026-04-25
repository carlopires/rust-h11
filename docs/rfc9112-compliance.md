# RFC 9112 Compliance Notes

This is a working checklist for HTTP/1.1 message syntax, parsing, framing, and
connection management. It is intentionally scoped to h11's Sans-I/O protocol
core; transport, TLS, caching, authentication, routing, and application
semantics belong outside this crate or in RFC 9110.

## Covered

- Section 2.1 message format: parses start-line, header section, blank-line
  terminator, and optional body as bytes.
- Section 2.3 HTTP version: accepts HTTP/1.x start lines and tracks peer
  version for connection behavior.
- Section 3 request line: validates method token and request target bytes.
- Section 4 status line: validates status code, reason phrase, and HTTP
  version through `Response::new` and writer validation.
- Section 5 field syntax: validates field names and field values, normalizes
  field names for lookup, and preserves raw field names for serialization.
- Section 5.2 obsolete line folding: unfolds received obs-fold continuation
  lines before header decoding.
- Section 6 message body: supports `Content-Length`, `Transfer-Encoding:
  chunked`, no-body responses, HEAD responses, CONNECT success responses, and
  close-delimited HTTP/1.0 bodies.
- Section 6.2 Content-Length: rejects invalid values and conflicting duplicate
  values.
- Section 7.1 chunked transfer coding: parses and serializes chunked bodies,
  including trailer fields.
- Section 9.3 persistence and Section 9.3.2 pipelining: tracks keep-alive,
  close, reusable connection cycles, and pipelined receive buffering.
- Section 11 security considerations: includes an incomplete-event buffer cap
  and rejects conflicting content lengths to reduce request smuggling risk.

## Partial

- Section 3.2 request target forms: validates target character shape but does
  not classify origin-form, absolute-form, authority-form, or asterisk-form.
- Section 6.1 Transfer-Encoding: supports only `chunked`; unsupported transfer
  codings are rejected with 501.
- Section 7.1.1 chunk extensions: parses and ignores chunk extensions.
- Section 7.1.2 trailer fields: parses trailers but does not enforce a denylist
  for fields that are invalid in trailers.
- Section 9.7 and Section 9.8 TLS connection behavior: out of scope for the
  Sans-I/O core.

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
- Add corpus fixtures for known request-smuggling and response-splitting cases.
- Expand this checklist as behavior is reviewed against RFC 9112 section by
  section.
