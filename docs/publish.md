# Publish Checklist

Use this checklist for the initial crates.io publish and later releases.

## Required Checks

```bash
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps
cargo check --manifest-path fuzz/Cargo.toml --all-targets
cargo package --list
cargo publish --dry-run
```

If `cargo-fuzz` is installed, run at least short release-candidate fuzz
sessions:

```bash
cargo fuzz run parse_request_stream -- -max_total_time=300
cargo fuzz run parse_response_stream -- -max_total_time=300
cargo fuzz run chunked_body_stream -- -max_total_time=300
cargo fuzz run state_machine_roundtrip -- -max_total_time=300
```

## Package Audit

Confirm `cargo package --list` includes:

- `src/`
- `tests/`
- `examples/`
- `benches/`
- `docs/`
- `README.md`
- `LICENSE`
- `CHANGELOG.md`
- `CONTRIBUTING.md`
- `SECURITY.md`

Confirm it excludes:

- `drafts/`
- `fuzz/`
- `target/`
- generated Criterion output
- local crash artifacts

## Release Notes

Before publishing:

- Update `CHANGELOG.md`.
- Confirm `Cargo.toml` version and `PRODUCT_ID` version are intentional.
- Confirm `rust-version` matches CI and the MSRV policy.
- Confirm docs.rs metadata, repository, license, keywords, and categories are
  accurate.
- Confirm CI is green on `main`.
