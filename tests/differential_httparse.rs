use h11::{Connection, Event, Role};

fn h11_request(data: &[u8]) -> Event {
    let mut conn = Connection::new(Role::Server, None);
    conn.receive_data(data).unwrap();
    conn.next_event().unwrap()
}

fn h11_response(data: &[u8]) -> Event {
    let mut conn = Connection::new(Role::Client, None);
    conn.receive_data(data).unwrap();
    conn.next_event().unwrap()
}

#[test]
fn request_start_line_matches_httparse() {
    let cases = [
        b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n".as_slice(),
        b"POST /submit?x=1 HTTP/1.1\r\nHost: example.com\r\nContent-Length: 0\r\n\r\n",
        b"GET /old HTTP/1.0\r\nUser-Agent: fixture\r\n\r\n",
    ];

    for data in cases {
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut parsed = httparse::Request::new(&mut headers);
        assert!(parsed.parse(data).unwrap().is_complete());

        match h11_request(data) {
            Event::Request(request) => {
                assert_eq!(request.method, parsed.method.unwrap().as_bytes());
                assert_eq!(request.target, parsed.path.unwrap().as_bytes());
                assert_eq!(
                    request.http_version,
                    format!("1.{}", parsed.version.unwrap()).as_bytes()
                );
            }
            event => panic!("unexpected h11 event: {event:?}"),
        }
    }
}

#[test]
fn response_start_line_matches_httparse() {
    let cases = [
        b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n".as_slice(),
        b"HTTP/1.1 204 No Content\r\n\r\n",
        b"HTTP/1.0 404 Not Found\r\nConnection: close\r\n\r\n",
    ];

    for data in cases {
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut parsed = httparse::Response::new(&mut headers);
        assert!(parsed.parse(data).unwrap().is_complete());

        match h11_response(data) {
            Event::NormalResponse(response) | Event::InformationalResponse(response) => {
                assert_eq!(response.status_code, parsed.code.unwrap());
                assert_eq!(response.reason, parsed.reason.unwrap_or("").as_bytes());
                assert_eq!(
                    response.http_version,
                    format!("1.{}", parsed.version.unwrap()).as_bytes()
                );
            }
            event => panic!("unexpected h11 event: {event:?}"),
        }
    }
}

#[test]
fn request_rejections_match_httparse_for_malformed_start_lines() {
    let cases = [
        b"G ET / HTTP/1.1\r\nHost: example.com\r\n\r\n".as_slice(),
        b"GET / HTTX/1.1\r\nHost: example.com\r\n\r\n",
        b"GET / HTTP/1.A\r\nHost: example.com\r\n\r\n",
    ];

    for data in cases {
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut parsed = httparse::Request::new(&mut headers);
        assert!(parsed.parse(data).is_err());

        let mut conn = Connection::new(Role::Server, None);
        conn.receive_data(data).unwrap();
        assert!(conn.next_event().is_err());
    }
}
