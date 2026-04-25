# Performance Baselines

This page records local parser and serializer benchmark baselines used to
evaluate parser refactors. Treat these numbers as comparison points for the
same machine and toolchain, not as portable performance claims.

## Running Benchmarks

Use the full Criterion run when comparing a parser change:

```bash
cargo bench --bench parser -- --noplot
```

Use the quick run only for smoke checks while iterating:

```bash
cargo bench --bench parser -- --quick --noplot
```

Generated Criterion artifacts stay under `target/` and are not committed.

## Baseline: 2026-04-25

- Git commit: `dc86745`
- Command: `cargo bench --bench parser -- --quick --noplot`
- Rust: `rustc 1.94.1 (e408947bf 2026-03-25)`
- Host: `aarch64-apple-darwin`
- OS: `Darwin 24.3.0 arm64`

| Benchmark | Median time | Median throughput |
| --- | ---: | ---: |
| `parse/small_get_request` | 4.1309 us | 12.928 MiB/s |
| `parse/large_header_request` | 131.35 us | 51.346 MiB/s |
| `parse/chunked_response` | 5.2929 us | 15.856 MiB/s |
| `parse/pipelined_requests` | 13.815 us | 8.4218 MiB/s |
| `serialize/response_content_length` | 7.0795 us | 1.7512 MiB/s |

Before replacing regex-based parsing, run a full benchmark on the pre-change
commit and compare it with the candidate parser implementation. Prefer a
same-machine comparison over comparing against the table above.
