use h11::{Connection, Event, Role};
use serde_json::Value;

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

fn role_from_fixture(value: &Value) -> Role {
    match value["role"].as_str().unwrap() {
        "CLIENT" => Role::Client,
        "SERVER" => Role::Server,
        role => panic!("unknown role: {role}"),
    }
}

fn rust_events_for_fixture(fixture: &Value) -> Vec<Value> {
    let mut conn = Connection::new(role_from_fixture(fixture), None);
    let mut events = Vec::new();

    for chunk in fixture["chunks_hex"].as_array().unwrap() {
        conn.receive_data(&decode_hex(chunk.as_str().unwrap()))
            .unwrap();
        loop {
            match conn.next_event().unwrap() {
                Event::NeedData() | Event::Paused() => break,
                event @ Event::ConnectionClosed(_) => {
                    events.push(event_json(event));
                    break;
                }
                event => events.push(event_json(event)),
            }
        }
    }

    events
}

fn assert_fixture_matches_python_h11(fixture_json: &str) {
    let fixture: Value = serde_json::from_str(fixture_json).unwrap();
    let expected = fixture["events"].as_array().unwrap().clone();
    assert_eq!(rust_events_for_fixture(&fixture), expected, "{fixture:#?}");
}

#[test]
fn python_h11_server_get_empty() {
    assert_fixture_matches_python_h11(include_str!("fixtures/python-h11/server_get_empty.json"));
}

#[test]
fn python_h11_server_post_content_length() {
    assert_fixture_matches_python_h11(include_str!(
        "fixtures/python-h11/server_post_content_length.json"
    ));
}

#[test]
fn python_h11_server_post_chunked_trailer() {
    assert_fixture_matches_python_h11(include_str!(
        "fixtures/python-h11/server_post_chunked_trailer.json"
    ));
}

#[test]
fn python_h11_client_response_content_length() {
    assert_fixture_matches_python_h11(include_str!(
        "fixtures/python-h11/client_response_content_length.json"
    ));
}

#[test]
fn python_h11_client_response_chunked_trailer() {
    assert_fixture_matches_python_h11(include_str!(
        "fixtures/python-h11/client_response_chunked_trailer.json"
    ));
}

#[test]
fn python_h11_client_http10_close_delimited() {
    assert_fixture_matches_python_h11(include_str!(
        "fixtures/python-h11/client_http10_close_delimited.json"
    ));
}
