# Security Policy

`h11` parses HTTP/1.1 bytes supplied by callers. Parser panics, request
smuggling behavior, response splitting behavior, and state-machine confusion can
be security-sensitive even though this crate does not perform network I/O.

## Supported Versions

Until the crate is published, security fixes target `main`. After publishing,
supported versions will be listed here.

| Version | Supported |
| --- | --- |
| `0.1.x` | yes after first publish |

## Reporting a Vulnerability

If this repository has GitHub private vulnerability reporting enabled, use that
channel. Otherwise, open a minimal public issue if disclosure is safe, or
contact the maintainer privately through the repository owner profile.

Include:

- A minimized input or event sequence.
- Expected behavior.
- Actual behavior.
- Whether the issue is a panic, protocol mismatch, resource exhaustion concern,
  or parser ambiguity.

## Handling

Security fixes should include a regression test. For parser inputs, prefer a
minimized fixture under `tests/fixtures/python-h11/` when Python h11 behavior is
relevant, or a focused Rust regression test otherwise. Fuzz crashes should be
minimized and promoted into the normal test suite.
