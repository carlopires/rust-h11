//! Sans-I/O HTTP/1.1 protocol handling.
//!
//! This crate parses and serializes HTTP/1.1 events without owning sockets,
//! timers, or tasks. Applications feed bytes into a [`Connection`] with
//! [`Connection::receive_data`], pull protocol events with
//! [`Connection::next_event`], and serialize outbound events with
//! [`Connection::send`].
//!
//! The API follows the same model as Python h11: callers drive a state machine
//! by exchanging typed events such as [`Request`], [`Response`], [`Data`], and
//! [`EndOfMessage`]. Invalid local usage returns [`LocalProtocolError`], while
//! malformed peer input returns [`RemoteProtocolError`].
//!
//! Use the fallible constructors such as [`Request::new_http11`],
//! [`Response::new_final_http11`], and [`Headers::new`] for public inputs.
//! Struct fields are currently public for compatibility, but manually-built
//! values should be validated before being sent.

#![allow(
    clippy::byte_char_slices,
    clippy::collapsible_if,
    clippy::for_kv_map,
    clippy::len_zero,
    clippy::match_like_matches_macro,
    clippy::needless_return,
    clippy::ptr_arg,
    clippy::redundant_pattern_matching,
    clippy::type_complexity,
    clippy::unit_arg,
    clippy::unnecessary_mut_passed,
    clippy::unnecessary_to_owned,
    clippy::unnecessary_unwrap,
    clippy::useless_conversion,
    clippy::useless_vec,
    clippy::while_let_loop,
    clippy::while_let_on_iterator
)]
mod _abnf;
mod _connection;
mod _events;
mod _headers;
mod _readers;
mod _receivebuffer;
mod _state;
mod _util;
mod _writers;

pub use _connection::Connection;
pub use _events::{ConnectionClosed, Data, EndOfMessage, Event, Request, Response};
pub use _headers::Headers;
pub use _state::{EventType, Role, State, Switch};
pub use _util::{LocalProtocolError, ProtocolError, RemoteProtocolError};
