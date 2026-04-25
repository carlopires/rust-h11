# Parser and Dependency Review

This note records the publish-readiness decision for the current regex-based
parser implementation.

## Decision

Keep `lazy_static` and `regex` for the initial `0.1.0` publish.

The crate now has:

- RFC 9112 coverage notes for parser and framing behavior.
- Pinned Python h11 differential fixtures for core flows and minimized
  malformed cases.
- Fuzz targets and seed corpora for request streams, response streams, chunked
  bodies, and state-machine roundtrips.
- Criterion benchmarks and a local baseline for representative parser and
  serializer hot paths.

That is enough safety infrastructure to publish without doing a risky parser
rewrite first. Replacing regex should be justified by measured performance,
clearer error behavior, or meaningful dependency reduction, not by preference.

## Before Replacing Regex

Run:

```bash
cargo test --all-targets
cargo bench --bench parser -- --noplot
cargo check --manifest-path fuzz/Cargo.toml --all-targets
```

Then compare the candidate parser against the documented baseline in
`docs/performance.md` using the same machine and toolchain.

## Acceptance Criteria for a Parser Rewrite

- Python h11 fixture comparisons remain green.
- RFC 9112 compliance notes stay accurate.
- Fuzz targets compile and short local fuzz runs do not find crashes.
- Benchmarks are neutral or better for representative hot paths, or the
  readability/security benefit is explicit enough to justify neutral/slower
  results.
- Any dependency removal is reflected in `Cargo.toml`, `Cargo.lock`, and
  release notes.
