# Hardening Guide

This crate parses hostile HTTP/1.1 bytes supplied by callers. Treat parser,
framing, and state-machine changes as security-sensitive work even though the
crate does not own sockets or timeouts.

## Local Fuzz Runs

Install cargo-fuzz once:

```bash
cargo install cargo-fuzz
```

Run the request and response stream targets before release candidates and after
parser/state-machine refactors:

```bash
cargo fuzz run parse_request_stream -- -max_total_time=300
cargo fuzz run parse_response_stream -- -max_total_time=300
```

Use longer runs before publishing or after risky parser changes:

```bash
cargo fuzz run parse_request_stream -- -max_total_time=3600
cargo fuzz run parse_response_stream -- -max_total_time=3600
```

Generated fuzz artifacts stay under `fuzz/target/` and `fuzz/artifacts/`; do
not commit them.

## Regression Workflow

When fuzzing finds a crash or panic:

1. Minimize the input with `cargo fuzz tmin`.
2. Reproduce the failure against the current branch.
3. Add the minimized input as a unit, integration, or Python h11 differential
   fixture when Python h11 behavior is relevant.
4. Fix the panic or protocol mismatch.
5. Keep the regression test in the normal `cargo test --all-targets` path.

For malformed peer bytes, public APIs should return
`ProtocolError::RemoteProtocolError`. For invalid local event construction or
send sequencing, APIs should return `ProtocolError::LocalProtocolError`.
Expected misuse and hostile input should not panic.

## Release Hardening Checklist

Before publishing a release:

```bash
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps
cargo package --allow-dirty
cargo publish --dry-run
cargo check --manifest-path fuzz/Cargo.toml --all-targets
```

Also run at least short fuzz sessions for every fuzz target and record the
commands in the release notes or pull request.

## Current Coverage

- Unit tests cover state transitions, body framing, chunked parsing/writing,
  keep-alive, protocol switching, and public panic regressions.
- `tests/differential_httparse.rs` compares request and response start-line
  handling with `httparse`.
- `tests/fixtures/python-h11/*.json` compares event streams with pinned Python
  h11 fixtures for valid flows and minimized malformed cases.
- `fuzz/corpus/` seeds request smuggling, duplicate `Content-Length`, malformed
  chunks, obs-fold, incomplete bodies, pipelining, and response-splitting cases.
