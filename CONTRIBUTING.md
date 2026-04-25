# Contributing

This crate is a Sans-I/O HTTP/1.1 protocol core. Keep changes focused on
protocol parsing, serialization, state transitions, public API ergonomics, docs,
or hardening. Transport, TLS, retries, timeouts, and connection pools belong in
callers or higher-level crates.

## Local Checks

Run these before opening a pull request:

```bash
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps
cargo check --manifest-path fuzz/Cargo.toml --all-targets
```

For packaging changes, also run:

```bash
cargo package --allow-dirty
cargo publish --dry-run
```

## Protocol Changes

- Add or update tests for every behavior change.
- Prefer returning `ProtocolError` over panicking for expected misuse or
  malformed peer input.
- Update `docs/rfc9112-compliance.md` when changing behavior tied to RFC 9112.
- Update Python h11 fixtures when parity behavior changes:

```bash
python3 -m venv /tmp/h11-fixtures
/tmp/h11-fixtures/bin/python -m pip install h11==0.16.0
/tmp/h11-fixtures/bin/python scripts/generate_python_h11_fixtures.py
```

## MSRV

The minimum supported Rust version is the `rust-version` declared in
`Cargo.toml`. CI should use the same toolchain. MSRV increases are breaking
changes for published releases and must be noted in `CHANGELOG.md`.

## Versioning

The crate uses semantic versioning. Before `1.0.0`, breaking public API changes
are allowed but should still be documented in `CHANGELOG.md` and kept
intentional.
