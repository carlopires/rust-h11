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
| `Request` | `Request` | partial | Fallible constructor accepts borrowed byte-like values. `new_http11` provides the common HTTP/1.1 default; Rust still uses positional arguments instead of Python keyword-only construction. |
| `InformationalResponse` | `Event::InformationalResponse(Response)` | partial | Represented as an enum variant. `Response::new_informational*` and `Event::informational_response` provide range-checked construction. |
| `Response` | `Response` / `Event::NormalResponse` | partial | Fallible constructor accepts borrowed byte-like values. `Response::new_final*` and `Event::normal_response` provide final-response range checks, but final responses are not separated as a distinct Rust type. |
| `Data` | `Data` | partial | Basic body chunks exist. Python supports sendfile-oriented data objects via `combine=False`; Rust currently requires owned `Vec<u8>`. |
| `EndOfMessage` | `EndOfMessage` | same | End event and trailer headers exist. |
| `ConnectionClosed` | `ConnectionClosed` | same | Close event exists. |
| `ProtocolError` | `ProtocolError` | partial | Local/remote variants exist and implement standard Rust error traits; exception-style inheritance does not apply in Rust. |
| `LocalProtocolError` | `LocalProtocolError` | partial | Exists with message and status code. |
| `RemoteProtocolError` | `RemoteProtocolError` | partial | Exists with message and status code. |
| `Headers` | `Headers` | partial | Normalization and raw casing preservation exist. Constructors accept borrowed byte-like values, but iteration clones values. |
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
| Accept `str`, bytes-like, and existing headers in event constructors | partial | Rust constructors accept `AsRef<[u8]>` inputs and existing `Headers`. They do not perform Python-style text encoding beyond byte copying. |
| Default `http_version` to `b"1.1"` for manual events | partial | `new_http11` convenience constructors exist. The fully explicit constructors still require a version. |
| Validate manually-created events at construction | partial | `Request::new` and `Response::new` validate; `Data`, `EndOfMessage`, and direct struct literals bypass checks. |
| Preserve raw header casing while exposing lowercase lookup names | same | Implemented by `Headers`. |
| Reject leading/trailing whitespace in header names | same | Implemented by validation. |
| Reject invalid `Content-Length` and conflicting duplicates | same | Implemented by validation. |
| Distinguish informational vs final response construction | partial | Range-checked constructors exist for informational and final responses. `Response` remains a shared struct for compatibility. |

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
| Extensive API documentation | partial | Public rustdoc covers the exported API surface; longer guide pages are still missing. |
| Exhaustive test suite / coverage target | partial | Unit and integration tests exist; no coverage target or branch coverage gate. |
| Fuzzing infrastructure | partial | cargo-fuzz harnesses and seed corpora exist; scheduled runs and crash regression promotion are still missing. |
| Differential behavior confidence | partial | `httparse` differential tests and pinned Python h11 JSON fixture comparisons exist for core request/response flows. Broader generated fixtures are still missing. |
| No runtime dependencies outside standard library | partial | Runtime depends on `lazy_static` and `regex`. Decide whether to keep or replace with byte parsers. |

## Initial Implementation Backlog

1. Expand Python h11 fixtures to cover malformed inputs, pipelining, `100-continue`, CONNECT, and Upgrade.
2. Audit remaining public panic paths and convert them to protocol errors.
3. Add longer guide pages for clients, servers, bodies, keep-alive, and protocol switching.
4. Decide whether `PRODUCT_ID` belongs in the Rust public API.
5. Benchmark parser hot paths before replacing regex-based parsing.
