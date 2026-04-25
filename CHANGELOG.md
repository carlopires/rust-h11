# Changelog

All notable changes to this crate are documented here.

This project follows semantic versioning once published. While the version is
`0.y.z`, public API changes can still be breaking, but they should be called
out explicitly in this file.

## 0.1.0 - Unreleased

Initial publish candidate.

### Added

- `rust-h11` package publishing the `h11` library crate.
- Sans-I/O HTTP/1.1 connection state machine.
- Request, response, data, end-of-message, and connection-closed events.
- Fallible constructors for requests, responses, and headers.
- Checked informational and final response constructors.
- `PRODUCT_ID` for optional `User-Agent` or `Server` identification.
- Python h11 differential fixture tests.
- RFC 9112 compliance notes.
- Fuzz targets and seed corpora for parser and state-machine hardening.
- User guide, cookbook examples, performance baselines, and hardening guide.

### Known Gaps

- Runtime still depends on `lazy_static` and `regex`.
- Request-target forms are validated but not classified.
- Trailer field-specific restrictions are not enforced.
- Coverage thresholds and scheduled fuzzing are not yet configured.
