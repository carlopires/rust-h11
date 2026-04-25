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


def run_case(case: dict[str, object]) -> dict[str, object]:
    role = getattr(h11, str(case["role"]))
    conn = h11.Connection(role)
    events = []

    for chunk in case["chunks"]:
        conn.receive_data(chunk)
        for _ in range(32):
            event = conn.next_event()
            if event in (h11.NEED_DATA, h11.PAUSED):
                break
            events.append(event_json(event))
            if isinstance(event, h11.ConnectionClosed):
                break
        else:
            raise RuntimeError(f"fixture did not quiesce: {case['name']}")

    return {
        "name": case["name"],
        "python_h11_version": h11.__version__,
        "role": case["role"],
        "chunks_hex": [hex_bytes(chunk) for chunk in case["chunks"]],
        "events": events,
    }


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    for case in CASES:
        fixture = run_case(case)
        path = OUT / f"{fixture['name']}.json"
        path.write_text(json.dumps(fixture, indent=2, sort_keys=True) + "\n")


if __name__ == "__main__":
    main()
