use h11::{Connection, EndOfMessage, Event, Headers, ProtocolError, Request, Role};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn decode_hex(hex: &str) -> Vec<u8> {
    assert_eq!(hex.len() % 2, 0, "hex input must have an even length");
    (0..hex.len())
        .step_by(2)
        .map(|idx| u8::from_str_radix(&hex[idx..idx + 2], 16).unwrap())
        .collect()
}

fn headers_json(headers: &h11::Headers) -> Value {
    Value::Array(
        headers
            .raw_items()
            .into_iter()
            .map(|(name, _, value)| {
                serde_json::json!({
                    "name_hex": name.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
                    "value_hex": value.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
                })
            })
            .collect(),
    )
}

fn bytes_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

fn event_json(event: Event) -> Value {
    match event {
        Event::Request(request) => serde_json::json!({
            "type": "Request",
            "method_hex": bytes_hex(&request.method),
            "target_hex": bytes_hex(&request.target),
            "http_version_hex": bytes_hex(&request.http_version),
            "headers": headers_json(&request.headers),
        }),
        Event::InformationalResponse(response) => serde_json::json!({
            "type": "InformationalResponse",
            "status_code": response.status_code,
            "reason_hex": bytes_hex(&response.reason),
            "http_version_hex": bytes_hex(&response.http_version),
            "headers": headers_json(&response.headers),
        }),
        Event::NormalResponse(response) => serde_json::json!({
            "type": "NormalResponse",
            "status_code": response.status_code,
            "reason_hex": bytes_hex(&response.reason),
            "http_version_hex": bytes_hex(&response.http_version),
            "headers": headers_json(&response.headers),
        }),
        Event::Data(data) => serde_json::json!({
            "type": "Data",
            "data_hex": bytes_hex(&data.data),
            "chunk_start": data.chunk_start,
            "chunk_end": data.chunk_end,
        }),
        Event::EndOfMessage(end_of_message) => serde_json::json!({
            "type": "EndOfMessage",
            "headers": headers_json(&end_of_message.headers),
        }),
        Event::ConnectionClosed(_) => serde_json::json!({ "type": "ConnectionClosed" }),
        Event::NeedData() => serde_json::json!({ "type": "NeedData" }),
        Event::Paused() => serde_json::json!({ "type": "Paused" }),
    }
}

fn error_json(error: ProtocolError) -> Value {
    match error {
        ProtocolError::RemoteProtocolError(error) => {
            serde_json::json!({ "type": "RemoteProtocolError", "code": error.code })
        }
        ProtocolError::LocalProtocolError(error) => {
            serde_json::json!({ "type": "LocalProtocolError", "code": error.code })
        }
    }
}

fn role_from_fixture(value: &Value) -> Role {
    match value["role"].as_str().unwrap() {
        "CLIENT" => Role::Client,
        "SERVER" => Role::Server,
        role => panic!("unknown role: {role}"),
    }
}

fn headers_from_json(value: &Value) -> Headers {
    Headers::new(
        value
            .as_array()
            .unwrap()
            .iter()
            .map(|header| {
                (
                    decode_hex(header["name_hex"].as_str().unwrap()),
                    decode_hex(header["value_hex"].as_str().unwrap()),
                )
            })
            .collect::<Vec<_>>(),
    )
    .unwrap()
}

fn apply_fixture_setup(conn: &mut Connection, fixture: &Value) {
    if let Some(setup_request) = fixture.get("setup_request") {
        let request = Request::new_http11(
            decode_hex(setup_request["method_hex"].as_str().unwrap()),
            headers_from_json(&setup_request["headers"]),
            decode_hex(setup_request["target_hex"].as_str().unwrap()),
        )
        .unwrap();
        conn.send(request.into()).unwrap();
        conn.send(EndOfMessage::default().into()).unwrap();
    }
}

fn rust_events_for_fixture(fixture: &Value) -> Vec<Value> {
    let mut conn = Connection::new(role_from_fixture(fixture), None);
    let mut events = Vec::new();
    let include_terminal = fixture["include_terminal"].as_bool().unwrap_or(false);
    apply_fixture_setup(&mut conn, fixture);

    for chunk in fixture["chunks_hex"].as_array().unwrap() {
        conn.receive_data(&decode_hex(chunk.as_str().unwrap()))
            .unwrap();
        loop {
            match conn.next_event() {
                Ok(event @ (Event::NeedData() | Event::Paused())) => {
                    if include_terminal {
                        events.push(event_json(event));
                    }
                    break;
                }
                Ok(event @ Event::ConnectionClosed(_)) => {
                    events.push(event_json(event));
                    break;
                }
                Ok(event) => events.push(event_json(event)),
                Err(error) => {
                    events.push(error_json(error));
                    return events;
                }
            }
        }
    }

    if let Some(expected) = fixture["they_are_waiting_for_100_continue"].as_bool() {
        assert_eq!(
            conn.get_they_are_waiting_for_100_continue(),
            expected,
            "{fixture:#?}"
        );
    }

    events
}

fn assert_fixture_matches_python_h11(fixture_json: &str, fixture_name: &str) {
    let fixture: Value = serde_json::from_str(fixture_json).unwrap();
    let expected = fixture["events"].as_array().unwrap().clone();
    assert_eq!(
        rust_events_for_fixture(&fixture),
        expected,
        "{fixture_name}: {fixture:#?}"
    );
}

#[test]
fn python_h11_fixtures_match() {
    let mut fixtures: Vec<PathBuf> = fs::read_dir(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/python-h11"
    ))
    .unwrap()
    .map(|entry| entry.unwrap().path())
    .filter(|path| {
        path.extension()
            .is_some_and(|extension| extension == "json")
    })
    .collect();
    fixtures.sort();

    for fixture in fixtures {
        let fixture_json = fs::read_to_string(&fixture).unwrap();
        assert_fixture_matches_python_h11(&fixture_json, &fixture.display().to_string());
    }
}
