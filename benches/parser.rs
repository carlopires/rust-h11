use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BatchSize, Criterion, Throughput};
use h11::{Connection, Data, EndOfMessage, Event, Headers, Response, Role};

fn drain_until_idle(conn: &mut Connection) -> usize {
    let mut events = 0;
    loop {
        match conn.next_event().unwrap() {
            Event::NeedData() | Event::Paused() => break,
            Event::Data(data) => {
                events += data.data.len();
            }
            _ => {
                events += 1;
            }
        }
    }
    events
}

fn drain_pipelined_requests(conn: &mut Connection) -> usize {
    let mut events = 0;
    loop {
        match conn.next_event().unwrap() {
            Event::EndOfMessage(_) => {
                events += 1;
                if conn.get_trailing_data().0.is_empty() {
                    break;
                }
                let response = Response::new_final_http11(
                    204,
                    Headers::new([("Content-Length", "0")]).unwrap(),
                    "No Content",
                )
                .unwrap();
                black_box(conn.send(response.into()).unwrap());
                black_box(conn.send(EndOfMessage::default().into()).unwrap());
                conn.start_next_cycle().unwrap();
            }
            Event::NeedData() | Event::Paused() => break,
            Event::Data(data) => {
                events += data.data.len();
            }
            _ => {
                events += 1;
            }
        }
    }
    events
}

fn large_header_request() -> Vec<u8> {
    let mut request = b"GET /large HTTP/1.1\r\nHost: example.com\r\n".to_vec();
    for idx in 0..64 {
        request.extend_from_slice(format!("X-Bench-{idx}: {}\r\n", "a".repeat(96)).as_bytes());
    }
    request.extend_from_slice(b"\r\n");
    request
}

fn pipelined_requests() -> Vec<u8> {
    [
        b"GET /one HTTP/1.1\r\nHost: example.com\r\n\r\n".as_slice(),
        b"GET /two HTTP/1.1\r\nHost: example.com\r\n\r\n".as_slice(),
        b"GET /three HTTP/1.1\r\nHost: example.com\r\n\r\n".as_slice(),
    ]
    .concat()
}

fn benchmark_parsing(c: &mut Criterion) {
    let small_get = b"GET / HTTP/1.1\r\nHost: example.com\r\nUser-Agent: bench\r\n\r\n";
    let large_headers = large_header_request();
    let chunked_response = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nWiki\r\n5\r\npedia\r\n0\r\nX-Trailer: done\r\n\r\n";
    let pipelined = pipelined_requests();

    let mut group = c.benchmark_group("parse");

    group.throughput(Throughput::Bytes(small_get.len() as u64));
    group.bench_function("small_get_request", |b| {
        b.iter(|| {
            let mut conn = Connection::new(Role::Server, None);
            conn.receive_data(black_box(small_get)).unwrap();
            black_box(drain_until_idle(&mut conn));
        });
    });

    group.throughput(Throughput::Bytes(large_headers.len() as u64));
    group.bench_function("large_header_request", |b| {
        b.iter(|| {
            let mut conn = Connection::new(Role::Server, None);
            conn.receive_data(black_box(&large_headers)).unwrap();
            black_box(drain_until_idle(&mut conn));
        });
    });

    group.throughput(Throughput::Bytes(chunked_response.len() as u64));
    group.bench_function("chunked_response", |b| {
        b.iter(|| {
            let mut conn = Connection::new(Role::Client, None);
            conn.receive_data(black_box(chunked_response)).unwrap();
            black_box(drain_until_idle(&mut conn));
        });
    });

    group.throughput(Throughput::Bytes(pipelined.len() as u64));
    group.bench_function("pipelined_requests", |b| {
        b.iter(|| {
            let mut conn = Connection::new(Role::Server, None);
            conn.receive_data(black_box(&pipelined)).unwrap();
            black_box(drain_pipelined_requests(&mut conn));
        });
    });

    group.finish();
}

fn benchmark_serialization(c: &mut Criterion) {
    let response = Response::new_final_http11(
        200,
        Headers::new([
            ("Content-Length", "13"),
            ("Server", "rust-h11-bench"),
            ("Content-Type", "text/plain"),
        ])
        .unwrap(),
        "OK",
    )
    .unwrap();
    let body = Data {
        data: b"hello, world!".to_vec(),
        ..Default::default()
    };
    let eom = EndOfMessage::default();

    let mut group = c.benchmark_group("serialize");
    group.throughput(Throughput::Bytes(13));
    group.bench_function("response_content_length", |b| {
        b.iter_batched(
            || {
                (
                    Connection::new(Role::Server, None),
                    response.clone(),
                    body.clone(),
                    eom.clone(),
                )
            },
            |(mut conn, response, body, eom)| {
                conn.receive_data(b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n")
                    .unwrap();
                assert!(matches!(conn.next_event().unwrap(), Event::Request(_)));
                assert!(matches!(conn.next_event().unwrap(), Event::EndOfMessage(_)));

                let mut bytes = Vec::new();
                bytes.extend(conn.send(response.into()).unwrap().unwrap());
                bytes.extend(conn.send(body.into()).unwrap().unwrap());
                bytes.extend(conn.send(eom.into()).unwrap().unwrap());
                black_box(bytes);
            },
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(benches, benchmark_parsing, benchmark_serialization);
criterion_main!(benches);
