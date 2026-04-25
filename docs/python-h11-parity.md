# Python h11 Parity Matrix

This document tracks the gap between Python h11 0.16.x and this Rust crate.
The goal is not automatic 1:1 API cloning; the goal is to make every difference
explicit so Rust-specific choices are intentional.

Statuses:

- `same`: equivalent public capability exists.
- `rust-specific`: Rust API differs intentionally but covers the same need.
- `missing`: Python h11 capability has no public Rust equivalent.
- `partial`: public Rust equivalent exists but behavior or ergonomics differ.
- `omit`: intentionally out of scope.
- `needs design`: decide before implementing.

## Public Symbols

| Python h11 symbol | Rust equivalent | Status | Notes |
| --- | --- | --- | --- |
| `Connection` | `Connection` | partial | Core send/receive/state behavior exists. Rust API returns `Option<Vec<u8>>` from `send`; Python supports additional send ergonomics such as `combine`. |
| `Request` | `Request` | partial | Fallible constructor exists. Rust constructor requires positional `Vec<u8>` values and explicit `http_version`; Python defaults to HTTP/1.1 and accepts `str`/bytes-like values. |
| `InformationalResponse` | `Event::InformationalResponse(Response)` | partial | Represented as an enum variant, not a separate public event type with range-specific construction. |
| `Response` | `Response` / `Event::NormalResponse` | partial | Fallible constructor exists, but final-response range is not separated at the type level. |
| `Data` | `Data` | partial | Basic body chunks exist. Python supports sendfile-oriented data objects via `combine=False`; Rust currently requires owned `Vec<u8>`. |
| `EndOfMessage` | `EndOfMessage` | same | End event and trailer headers exist. |
| `ConnectionClosed` | `ConnectionClosed` | same | Close event exists. |
| `ProtocolError` | `ProtocolError` | partial | Local/remote variants exist; Rust error types do not implement `std::error::Error` or `Display` yet. |
| `LocalProtocolError` | `LocalProtocolError` | partial | Exists with message and status code. |
| `RemoteProtocolError` | `RemoteProtocolError` | partial | Exists with message and status code. |
| `Headers` | `Headers` | partial | Normalization and raw casing preservation exist. Iteration clones values and construction ergonomics are narrower than Python. |
| `PRODUCT_ID` | none | missing | Useful for default `User-Agent` / `Server` identification if this crate wants Python-style helper behavior. |

## Roles and States

| Python h11 symbol | Rust equivalent | Status | Notes |
| --- | --- | --- | --- |
| `CLIENT` | `Role::Client` | rust-specific | Rust enum is clearer than sentinel constant. |
| `SERVER` | `Role::Server` | rust-specific | Rust enum is clearer than sentinel constant. |
| `IDLE` | `State::Idle` | rust-specific | Rust enum variant. |
| `SEND_RESPONSE` | `State::SendResponse` | rust-specific | Rust enum variant. |
| `SEND_BODY` | `State::SendBody` | rust-specific | Rust enum variant. |
| `DONE` | `State::Done` | rust-specific | Rust enum variant. |
| `MUST_CLOSE` | `State::MustClose` | rust-specific | Rust enum variant. |
| `CLOSED` | `State::Closed` | rust-specific | Rust enum variant. |
| `ERROR` | `State::Error` | rust-specific | Rust enum variant. |
| `MIGHT_SWITCH_PROTOCOL` | `State::MightSwitchProtocol` | rust-specific | Rust enum variant. |
| `SWITCHED_PROTOCOL` | `State::SwitchedProtocol` | rust-specific | Rust enum variant. |
| `NEED_DATA` | `Event::NeedData()` | partial | Python uses a sentinel; Rust models it inside `Event`. |
| `PAUSED` | `Event::Paused()` | partial | Python uses a sentinel; Rust models it inside `Event`. |

## Connection API

| Python h11 API | Rust equivalent | Status | Notes |
| --- | --- | --- | --- |
| `Connection(our_role, max_incomplete_event_size=...)` | `Connection::new(Role, Option<usize>)` | partial | Equivalent capability; Rust should consider a builder or default argument style. |
| `receive_data(data)` | `receive_data(&[u8])` | same | Both feed raw bytes into the connection. |
| `next_event()` | `next_event()` | same | Both emit protocol events or need-data/paused sentinels. |
| `send(event)` | `send(Event)` | partial | Rust returns `Result<Option<Vec<u8>>, ProtocolError>`; Python returns bytes or `None` for `ConnectionClosed`. |
| `send_with_data_passthrough(event)` | none | missing | Relevant if supporting zero-copy/sendfile-style data. |
| `send_failed()` | `send_failed()` | same | Marks local send side as errored. |
| `start_next_cycle()` | `start_next_cycle()` | same | Reuses keep-alive connection when both sides are done. |
| `states` | `get_states()` | partial | Rust clones a `HashMap`; consider exposing a stable borrowed view or role-specific getters only. |
| `our_state` | `get_our_state()` | same | Equivalent getter. |
| `their_state` | `get_their_state()` | same | Equivalent getter. |
| `they_are_waiting_for_100_continue` | `get_they_are_waiting_for_100_continue()` | same | Equivalent getter. |
| `client_is_waiting_for_100_continue` | `get_client_is_waiting_for_100_continue()` | same | Equivalent getter. |
| `trailing_data` | `get_trailing_data()` | same | Equivalent trailing bytes and EOF status. |

## Event Construction and Normalization

| Behavior | Rust status | Notes |
| --- | --- | --- |
| Accept `str`, bytes-like, and existing headers in event constructors | missing | Rust constructors currently require owned `Vec<u8>` and `Headers`. Add generic `TryIntoBytes` or explicit helper constructors. |
| Default `http_version` to `b"1.1"` for manual events | missing | Rust constructors require explicit version. |
| Validate manually-created events at construction | partial | `Request::new` and `Response::new` validate; `Data`, `EndOfMessage`, and direct struct literals bypass checks. |
| Preserve raw header casing while exposing lowercase lookup names | same | Implemented by `Headers`. |
| Reject leading/trailing whitespace in header names | same | Implemented by validation. |
| Reject invalid `Content-Length` and conflicting duplicates | same | Implemented by validation. |
| Distinguish informational vs final response construction | partial | Current `From<Response> for Event` chooses variant by status code; no separate type-level range enforcement. |

## Protocol Behavior

| Area | Rust status | Notes |
| --- | --- | --- |
| Content-Length body framing | same | Implemented and tested. |
| Chunked transfer decoding/encoding | same | Implemented and tested. |
| HTTP/1.0 close-delimited bodies | same | Implemented and tested. |
| Keep-alive reuse | same | Implemented and tested. |
| Pipelining | same | Implemented and tested. |
| `100-continue` state | same | Implemented and tested. |
| CONNECT protocol switch | same | Implemented and tested. |
| Upgrade protocol switch | same | Implemented and tested. |
| Obsolete line folding | same | Implemented and tested. |
| Request-target form classification | missing | Rust validates character shape but does not classify origin/absolute/authority/asterisk forms. |
| Trailer field restrictions | partial | Trailers parse, but field-specific trailer denylist is not enforced. |

## Quality and Tooling

| Python h11 quality marker | Rust status | Notes |
| --- | --- | --- |
| Extensive API documentation | missing | README has a basic example; rustdoc coverage is minimal. |
| Exhaustive test suite / coverage target | partial | Unit and integration tests exist; no coverage target or branch coverage gate. |
| Fuzzing infrastructure | partial | cargo-fuzz harnesses exist; corpora and scheduled runs are still missing. |
| Differential behavior confidence | partial | `httparse` differential tests exist; Python h11 differential fixtures are missing. |
| No runtime dependencies outside standard library | partial | Runtime depends on `lazy_static` and `regex`. Decide whether to keep or replace with byte parsers. |

## Initial Implementation Backlog

1. Add public rustdoc for every exported type and method.
2. Add generic constructors for `Request`, `Response`, and `Headers`.
3. Add explicit constructors for informational and final responses.
4. Implement `Display` and `std::error::Error` for protocol errors.
5. Add Python h11 fixture generator and JSON event comparison tests.
6. Expand RFC 9112 compliance notes into a section-by-section table.
7. Add fuzz corpus seeds for smuggling, splitting, chunk, obs-fold, and EOF cases.
8. Audit remaining public panic paths and convert them to protocol errors.
9. Decide whether `PRODUCT_ID` belongs in the Rust public API.
10. Benchmark parser hot paths before replacing regex-based parsing.
