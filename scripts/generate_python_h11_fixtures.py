#!/usr/bin/env python3
"""Regenerate Python h11 event fixtures.

Usage:
    python3 -m venv /tmp/h11-fixtures
    /tmp/h11-fixtures/bin/python -m pip install h11==0.16.0
    /tmp/h11-fixtures/bin/python scripts/generate_python_h11_fixtures.py
"""

from __future__ import annotations

import json
from pathlib import Path

import h11


ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "tests" / "fixtures" / "python-h11"


CASES = [
    {
        "name": "server_get_empty",
        "role": "SERVER",
        "chunks": [b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n"],
    },
    {
        "name": "server_post_content_length",
        "role": "SERVER",
        "chunks": [
            b"POST /submit HTTP/1.1\r\n"
            b"Host: example.com\r\n"
            b"Content-Length: 5\r\n"
            b"\r\n"
            b"hello"
        ],
    },
    {
        "name": "server_post_chunked_trailer",
        "role": "SERVER",
        "chunks": [
            b"POST /upload HTTP/1.1\r\n"
            b"Host: example.com\r\n"
            b"Transfer-Encoding: chunked\r\n"
            b"\r\n"
            b"5\r\nhello\r\n0\r\nEtag: abc\r\n\r\n"
        ],
    },
    {
        "name": "client_response_content_length",
        "role": "CLIENT",
        "chunks": [b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello"],
    },
    {
        "name": "client_response_chunked_trailer",
        "role": "CLIENT",
        "chunks": [
            b"HTTP/1.1 200 OK\r\n"
            b"Transfer-Encoding: chunked\r\n"
            b"\r\n"
            b"5\r\nhello\r\n0\r\nEtag: abc\r\n\r\n"
        ],
    },
    {
        "name": "client_http10_close_delimited",
        "role": "CLIENT",
        "chunks": [b"HTTP/1.0 200 OK\r\n\r\nhello", b""],
    },
    {
        "name": "server_pipelined_requests",
        "role": "SERVER",
        "include_terminal": True,
        "chunks": [
            b"GET /one HTTP/1.1\r\n"
            b"Host: example.com\r\n"
            b"\r\n"
            b"GET /two HTTP/1.1\r\n"
            b"Host: example.com\r\n"
            b"\r\n"
        ],
    },
    {
        "name": "server_expect_100_continue_pending",
        "role": "SERVER",
        "include_terminal": True,
        "capture_waiting_for_100_continue": True,
        "chunks": [
            b"POST /upload HTTP/1.1\r\n"
            b"Host: example.com\r\n"
            b"Expect: 100-continue\r\n"
            b"Content-Length: 5\r\n"
            b"\r\n"
        ],
    },
    {
        "name": "client_informational_then_final",
        "role": "CLIENT",
        "setup_request": {
            "method": b"GET",
            "target": b"/",
            "headers": [(b"Host", b"example.com")],
        },
        "chunks": [
            b"HTTP/1.1 100 Continue\r\n"
            b"\r\n"
            b"HTTP/1.1 200 OK\r\n"
            b"Content-Length: 0\r\n"
            b"\r\n"
        ],
    },
    {
        "name": "server_connect_request",
        "role": "SERVER",
        "chunks": [b"CONNECT example.com:443 HTTP/1.1\r\nHost: example.com\r\n\r\n"],
    },
    {
        "name": "server_upgrade_request",
        "role": "SERVER",
        "chunks": [
            b"GET /chat HTTP/1.1\r\n"
            b"Host: example.com\r\n"
            b"Connection: Upgrade\r\n"
            b"Upgrade: websocket\r\n"
            b"\r\n"
        ],
    },
    {
        "name": "server_obs_fold_header",
        "role": "SERVER",
        "chunks": [
            b"GET /folded HTTP/1.1\r\n"
            b"Host: example.com\r\n"
            b"X-Folded: one\r\n"
            b"\t two\r\n"
            b"\r\n"
        ],
    },
    {
        "name": "client_obs_fold_header",
        "role": "CLIENT",
        "chunks": [
            b"HTTP/1.1 200 OK\r\n"
            b"X-Folded: one\r\n"
            b"  two\r\n"
            b"Content-Length: 0\r\n"
            b"\r\n"
        ],
    },
    {
        "name": "server_duplicate_content_length_same",
        "role": "SERVER",
        "chunks": [
            b"POST /upload HTTP/1.1\r\n"
            b"Host: example.com\r\n"
            b"Content-Length: 5\r\n"
            b"Content-Length: 5\r\n"
            b"\r\n"
            b"hello"
        ],
    },
    {
        "name": "client_duplicate_content_length_same",
        "role": "CLIENT",
        "chunks": [
            b"HTTP/1.1 200 OK\r\n"
            b"Content-Length: 5\r\n"
            b"Content-Length: 5\r\n"
            b"\r\n"
            b"hello"
        ],
    },
    {
        "name": "server_duplicate_content_length_conflict",
        "role": "SERVER",
        "chunks": [
            b"POST /upload HTTP/1.1\r\n"
            b"Host: example.com\r\n"
            b"Content-Length: 5\r\n"
            b"Content-Length: 6\r\n"
            b"\r\n"
            b"hello"
        ],
    },
    {
        "name": "client_duplicate_content_length_conflict",
        "role": "CLIENT",
        "chunks": [
            b"HTTP/1.1 200 OK\r\n"
            b"Content-Length: 5\r\n"
            b"Content-Length: 6\r\n"
            b"\r\n"
            b"hello"
        ],
    },
    {
        "name": "server_unsupported_transfer_coding",
        "role": "SERVER",
        "chunks": [
            b"POST /upload HTTP/1.1\r\n"
            b"Host: example.com\r\n"
            b"Transfer-Encoding: gzip\r\n"
            b"\r\n"
        ],
    },
    {
        "name": "client_unsupported_transfer_coding",
        "role": "CLIENT",
        "chunks": [b"HTTP/1.1 200 OK\r\nTransfer-Encoding: gzip\r\n\r\n"],
    },
    {
        "name": "server_malformed_header_line",
        "role": "SERVER",
        "chunks": [
            b"GET / HTTP/1.1\r\n"
            b"Host: example.com\r\n"
            b"Bad Header\r\n"
            b"\r\n"
        ],
    },
    {
        "name": "client_malformed_header_line",
        "role": "CLIENT",
        "chunks": [b"HTTP/1.1 200 OK\r\nBad Header\r\n\r\n"],
    },
    {
        "name": "server_continuation_header_at_start",
        "role": "SERVER",
        "chunks": [
            b"GET / HTTP/1.1\r\n"
            b"\tcontinued\r\n"
            b"Host: example.com\r\n"
            b"\r\n"
        ],
    },
    {
        "name": "client_continuation_header_at_start",
        "role": "CLIENT",
        "chunks": [b"HTTP/1.1 200 OK\r\n continued\r\nContent-Length: 0\r\n\r\n"],
    },
    {
        "name": "server_malformed_request_line",
        "role": "SERVER",
        "chunks": [b"GET / HTTP/1.1 BAD\r\nHost: example.com\r\n\r\n"],
    },
    {
        "name": "client_malformed_response_status_line",
        "role": "CLIENT",
        "chunks": [b"HTTP/1.1 OK\r\n\r\n"],
    },
    {
        "name": "server_incomplete_content_length_eof",
        "role": "SERVER",
        "chunks": [
            b"POST /upload HTTP/1.1\r\n"
            b"Host: example.com\r\n"
            b"Content-Length: 5\r\n"
            b"\r\n"
            b"he",
            b"",
        ],
    },
    {
        "name": "client_incomplete_content_length_eof",
        "role": "CLIENT",
        "chunks": [b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhe", b""],
    },
    {
        "name": "server_malformed_chunk_size",
        "role": "SERVER",
        "chunks": [
            b"POST /upload HTTP/1.1\r\n"
            b"Host: example.com\r\n"
            b"Transfer-Encoding: chunked\r\n"
            b"\r\n"
            b"z\r\n"
        ],
    },
    {
        "name": "client_malformed_chunk_size",
        "role": "CLIENT",
        "chunks": [b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\nz\r\n"],
    },
    {
        "name": "server_incomplete_chunked_eof",
        "role": "SERVER",
        "chunks": [
            b"POST /upload HTTP/1.1\r\n"
            b"Host: example.com\r\n"
            b"Transfer-Encoding: chunked\r\n"
            b"\r\n"
            b"5\r\nhe",
            b"",
        ],
    },
    {
        "name": "client_incomplete_chunked_eof",
        "role": "CLIENT",
        "chunks": [b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhe", b""],
    },
]


def hex_bytes(data: bytes) -> str:
    return data.hex()


def headers_json(headers: h11.Headers) -> list[dict[str, str]]:
    return [
        {"name_hex": hex_bytes(name), "value_hex": hex_bytes(value)}
        for name, value in headers.raw_items()
    ]


def event_json(event: object) -> dict[str, object]:
    if event is h11.NEED_DATA:
        return {"type": "NeedData"}
    if event is h11.PAUSED:
        return {"type": "Paused"}
    if isinstance(event, h11.Request):
        return {
            "type": "Request",
            "method_hex": hex_bytes(event.method),
            "target_hex": hex_bytes(event.target),
            "http_version_hex": hex_bytes(event.http_version),
            "headers": headers_json(event.headers),
        }
    if isinstance(event, h11.InformationalResponse):
        return {
            "type": "InformationalResponse",
            "status_code": event.status_code,
            "reason_hex": hex_bytes(event.reason),
            "http_version_hex": hex_bytes(event.http_version),
            "headers": headers_json(event.headers),
        }
    if isinstance(event, h11.Response):
        return {
            "type": "NormalResponse",
            "status_code": event.status_code,
            "reason_hex": hex_bytes(event.reason),
            "http_version_hex": hex_bytes(event.http_version),
            "headers": headers_json(event.headers),
        }
    if isinstance(event, h11.Data):
        return {
            "type": "Data",
            "data_hex": hex_bytes(event.data),
            "chunk_start": event.chunk_start,
            "chunk_end": event.chunk_end,
        }
    if isinstance(event, h11.EndOfMessage):
        return {"type": "EndOfMessage", "headers": headers_json(event.headers)}
    if isinstance(event, h11.ConnectionClosed):
        return {"type": "ConnectionClosed"}
    raise TypeError(f"unsupported event: {event!r}")


def error_json(error: h11.RemoteProtocolError) -> dict[str, object]:
    return {
        "type": "RemoteProtocolError",
        "code": error.error_status_hint,
    }


def setup_request_json(request: dict[str, object]) -> dict[str, object]:
    return {
        "method_hex": hex_bytes(request["method"]),
        "target_hex": hex_bytes(request["target"]),
        "headers": [
            {"name_hex": hex_bytes(name), "value_hex": hex_bytes(value)}
            for name, value in request["headers"]
        ],
    }


def run_case(case: dict[str, object]) -> dict[str, object]:
    role = getattr(h11, str(case["role"]))
    conn = h11.Connection(role)
    events = []
    errored = False
    include_terminal = bool(case.get("include_terminal", False))
    setup_request = case.get("setup_request")

    if setup_request is not None:
        conn.send(
            h11.Request(
                method=setup_request["method"],
                target=setup_request["target"],
                headers=setup_request["headers"],
            )
        )
        conn.send(h11.EndOfMessage())

    for chunk in case["chunks"]:
        conn.receive_data(chunk)
        for _ in range(32):
            try:
                event = conn.next_event()
            except h11.RemoteProtocolError as error:
                events.append(error_json(error))
                errored = True
                break
            if event in (h11.NEED_DATA, h11.PAUSED):
                if include_terminal:
                    events.append(event_json(event))
                break
            events.append(event_json(event))
            if isinstance(event, h11.ConnectionClosed):
                break
        else:
            raise RuntimeError(f"fixture did not quiesce: {case['name']}")
        if errored:
            break

    fixture = {
        "name": case["name"],
        "python_h11_version": h11.__version__,
        "role": case["role"],
        "chunks_hex": [hex_bytes(chunk) for chunk in case["chunks"]],
        "events": events,
    }
    if include_terminal:
        fixture["include_terminal"] = True
    if setup_request is not None:
        fixture["setup_request"] = setup_request_json(setup_request)
    if case.get("capture_waiting_for_100_continue", False):
        fixture["they_are_waiting_for_100_continue"] = (
            conn.they_are_waiting_for_100_continue
        )
    return fixture


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    for case in CASES:
        fixture = run_case(case)
        path = OUT / f"{fixture['name']}.json"
        path.write_text(json.dumps(fixture, indent=2, sort_keys=True) + "\n")


if __name__ == "__main__":
    main()
