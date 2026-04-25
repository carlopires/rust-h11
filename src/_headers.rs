use std::collections::HashSet;

use crate::{
    _abnf::{FIELD_NAME, FIELD_VALUE},
    _events::Request,
    _util::ProtocolError,
};
use lazy_static::lazy_static;
use regex::bytes::Regex;

lazy_static! {
    static ref CONTENT_LENGTH_RE: Regex = Regex::new(r"^[0-9]+$").unwrap();
    static ref FIELD_NAME_RE: Regex = Regex::new(&format!(r"^{}$", FIELD_NAME)).unwrap();
    static ref FIELD_VALUE_RE: Regex = Regex::new(&format!(r"^{}$", *FIELD_VALUE)).unwrap();
}

fn trim_ascii_whitespace(value: &[u8]) -> &[u8] {
    let start = value
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(value.len());
    let end = value
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map(|idx| idx + 1)
        .unwrap_or(start);
    &value[start..end]
}

/// HTTP header collection.
///
/// Header names are stored in normalized lowercase form for lookup, while the
/// original raw casing is retained for serialization.
#[derive(Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct Headers(Vec<(Vec<u8>, Vec<u8>, Vec<u8>)>);

impl std::fmt::Debug for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("Headers");
        self.0.iter().for_each(|(raw_name, _, value)| {
            debug_struct.field(
                &String::from_utf8_lossy(raw_name),
                &String::from_utf8_lossy(value),
            );
        });
        debug_struct.finish()
    }
}

impl Headers {
    /// Returns normalized `(name, value)` pairs.
    ///
    /// Names are lowercase. Values preserve their stored bytes.
    pub fn iter(&self) -> impl Iterator<Item = (Vec<u8>, Vec<u8>)> + '_ {
        self.0
            .iter()
            .map(|(_, name, value)| ((*name).clone(), (*value).clone()))
    }

    /// Returns raw `(raw_name, normalized_name, value)` header triples.
    pub fn raw_items(&self) -> Vec<&(Vec<u8>, Vec<u8>, Vec<u8>)> {
        self.0.iter().collect()
    }

    /// Returns the number of header fields.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true when the collection has no header fields.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Builds and validates a header collection from byte-like name/value pairs.
    ///
    /// This validates field syntax, normalizes names for lookup, preserves raw
    /// name casing for output, and enforces `Content-Length` /
    /// `Transfer-Encoding` consistency.
    pub fn new<I, N, V>(headers: I) -> Result<Self, ProtocolError>
    where
        I: IntoIterator<Item = (N, V)>,
        N: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        normalize_and_validate(
            headers
                .into_iter()
                .map(|(name, value)| (name.as_ref().to_vec(), value.as_ref().to_vec()))
                .collect(),
            false,
        )
    }
}

impl From<Vec<(Vec<u8>, Vec<u8>)>> for Headers {
    /// Builds headers from owned byte vectors.
    ///
    /// This conversion panics if the headers are invalid. Prefer
    /// [`Headers::new`] when handling untrusted or fallible input.
    fn from(value: Vec<(Vec<u8>, Vec<u8>)>) -> Self {
        Headers::new(value)
            .expect("invalid HTTP header list; use Headers::new for fallible construction")
    }
}

/// Normalizes and validates HTTP header fields.
///
/// This is primarily used by parsers and [`Headers::new`]. The `_parsed`
/// argument skips field syntax checks for already-parsed wire input.
pub fn normalize_and_validate(
    headers: Vec<(Vec<u8>, Vec<u8>)>,
    _parsed: bool,
) -> Result<Headers, ProtocolError> {
    let mut new_headers = vec![];
    let mut seen_content_length = None;
    let mut saw_transfer_encoding = false;
    for (name, value) in headers {
        if !_parsed {
            if !FIELD_NAME_RE.is_match(&name) {
                return Err(ProtocolError::LocalProtocolError(
                    format!("Illegal header name {:?}", &name).into(),
                ));
            }
            if !FIELD_VALUE_RE.is_match(&value) {
                return Err(ProtocolError::LocalProtocolError(
                    format!("Illegal header value {:?}", &value).into(),
                ));
            }
        }
        let raw_name = name.clone();
        let name = name.to_ascii_lowercase();
        if name == b"content-length" {
            let lengths: HashSet<Vec<u8>> = value
                .split(|&b| b == b',')
                .map(|length| trim_ascii_whitespace(length).to_vec())
                .collect();
            if lengths.len() != 1 {
                return Err(ProtocolError::LocalProtocolError(
                    "conflicting Content-Length headers".into(),
                ));
            }
            let value = lengths.iter().next().unwrap();
            if !CONTENT_LENGTH_RE.is_match(value) {
                return Err(ProtocolError::LocalProtocolError(
                    "bad Content-Length".into(),
                ));
            }
            if seen_content_length.is_none() {
                seen_content_length = Some(value.clone());
                new_headers.push((raw_name, name, value.clone()));
            } else if seen_content_length != Some(value.clone()) {
                return Err(ProtocolError::LocalProtocolError(
                    "conflicting Content-Length headers".into(),
                ));
            }
        } else if name == b"transfer-encoding" {
            // "A server that receives a request message with a transfer coding
            // it does not understand SHOULD respond with 501 (Not
            // Implemented)."
            // https://www.rfc-editor.org/rfc/rfc9112.html#section-6.1
            if saw_transfer_encoding {
                return Err(ProtocolError::LocalProtocolError(
                    ("multiple Transfer-Encoding headers", 501).into(),
                ));
            }
            // "All transfer-coding names are case-insensitive"
            // -- https://www.rfc-editor.org/rfc/rfc9112.html#section-7
            let value = value.to_ascii_lowercase();
            if value != b"chunked" {
                return Err(ProtocolError::LocalProtocolError(
                    ("Only Transfer-Encoding: chunked is supported", 501).into(),
                ));
            }
            saw_transfer_encoding = true;
            new_headers.push((raw_name, name, value));
        } else {
            new_headers.push((raw_name, name, value.to_vec()));
        }
    }

    Ok(Headers(new_headers))
}

/// Reads a comma-separated header value as lowercase trimmed byte values.
pub fn get_comma_header(headers: &Headers, name: &[u8]) -> Vec<Vec<u8>> {
    let mut out: Vec<Vec<u8>> = vec![];
    let name = name.to_ascii_lowercase();
    for (found_name, found_value) in headers.iter() {
        if found_name == name {
            for found_split_value in found_value.to_ascii_lowercase().split(|&b| b == b',') {
                let found_split_value = trim_ascii_whitespace(found_split_value);
                if !found_split_value.is_empty() {
                    out.push(found_split_value.to_vec());
                }
            }
        }
    }
    out
}

/// Replaces all instances of a comma-separated header.
pub fn set_comma_header(
    headers: &Headers,
    name: &[u8],
    new_values: Vec<Vec<u8>>,
) -> Result<Headers, ProtocolError> {
    let mut new_headers = vec![];
    for (found_name, found_value) in headers.iter() {
        if found_name != name {
            new_headers.push((found_name, found_value));
        }
    }
    for new_value in new_values {
        new_headers.push((name.to_vec(), new_value));
    }
    normalize_and_validate(new_headers, false)
}

/// Returns whether a request contains an active `Expect: 100-continue`.
pub fn has_expect_100_continue(request: &Request) -> bool {
    // https://www.rfc-editor.org/rfc/rfc9110.html#section-10.1.1
    // "A server that receives a 100-continue expectation in an HTTP/1.0 request
    // MUST ignore that expectation."
    if request.http_version < b"1.1".to_vec() {
        return false;
    }
    let expect = get_comma_header(&request.headers, b"expect");
    expect.contains(&b"100-continue".to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_headers_new_rejects_invalid_input() {
        assert!(Headers::new(vec![(b"bad header".to_vec(), b"value".to_vec())]).is_err());
    }

    #[test]
    fn test_non_utf8_comma_headers_do_not_panic() {
        assert_eq!(
            normalize_and_validate(vec![(b"Content-Length".to_vec(), b"\xff".to_vec())], true)
                .unwrap_err(),
            ProtocolError::LocalProtocolError("bad Content-Length".into())
        );

        let headers = normalize_and_validate(
            vec![(b"Connection".to_vec(), b"close, \xff".to_vec())],
            true,
        )
        .unwrap();
        assert_eq!(
            get_comma_header(&headers, b"connection"),
            vec![b"close".to_vec(), b"\xff".to_vec()]
        );
    }

    #[test]
    fn test_headers_new_accepts_borrowed_inputs() {
        assert_eq!(
            Headers::new([("Host", "example.com"), ("Accept", "*/*")]).unwrap(),
            Headers(vec![
                (b"Host".to_vec(), b"host".to_vec(), b"example.com".to_vec()),
                (b"Accept".to_vec(), b"accept".to_vec(), b"*/*".to_vec()),
            ])
        );
        assert_eq!(
            Headers::new([(b"Host".as_slice(), b"example.com".as_slice())]).unwrap(),
            Headers(vec![(
                b"Host".to_vec(),
                b"host".to_vec(),
                b"example.com".to_vec()
            )])
        );
    }

    #[test]
    fn test_normalize_and_validate() {
        assert_eq!(
            normalize_and_validate(vec![(b"foo".to_vec(), b"bar".to_vec())], false).unwrap(),
            Headers(vec![(b"foo".to_vec(), b"foo".to_vec(), b"bar".to_vec())])
        );

        // no leading/trailing whitespace in names
        assert_eq!(
            normalize_and_validate(vec![(b"foo ".to_vec(), b"bar".to_vec())], false)
                .expect_err("Expect ProtocolError::LocalProtocolError"),
            ProtocolError::LocalProtocolError(
                ("Illegal header name [102, 111, 111, 32]".to_string(), 400).into()
            )
        );
        assert_eq!(
            normalize_and_validate(vec![(b" foo".to_vec(), b"bar".to_vec())], false)
                .expect_err("Expect ProtocolError::LocalProtocolError"),
            ProtocolError::LocalProtocolError(
                ("Illegal header name [32, 102, 111, 111]".to_string(), 400).into()
            )
        );

        // no weird characters in names
        assert_eq!(
            normalize_and_validate(vec![(b"foo bar".to_vec(), b"baz".to_vec())], false)
                .expect_err("Expect ProtocolError::LocalProtocolError"),
            ProtocolError::LocalProtocolError(
                (
                    "Illegal header name [102, 111, 111, 32, 98, 97, 114]".to_string(),
                    400
                )
                    .into()
            )
        );
        assert_eq!(
            normalize_and_validate(vec![(b"foo\x00bar".to_vec(), b"baz".to_vec())], false)
                .expect_err("Expect ProtocolError::LocalProtocolError"),
            ProtocolError::LocalProtocolError(
                (
                    "Illegal header name [102, 111, 111, 0, 98, 97, 114]".to_string(),
                    400
                )
                    .into()
            )
        );
        // Not even 8-bit characters:
        assert_eq!(
            normalize_and_validate(vec![(b"foo\xffbar".to_vec(), b"baz".to_vec())], false)
                .expect_err("Expect ProtocolError::LocalProtocolError"),
            ProtocolError::LocalProtocolError(
                (
                    "Illegal header name [102, 111, 111, 255, 98, 97, 114]".to_string(),
                    400
                )
                    .into()
            )
        );
        // And not even the control characters we allow in values:
        assert_eq!(
            normalize_and_validate(vec![(b"foo\x01bar".to_vec(), b"baz".to_vec())], false)
                .expect_err("Expect ProtocolError::LocalProtocolError"),
            ProtocolError::LocalProtocolError(
                (
                    "Illegal header name [102, 111, 111, 1, 98, 97, 114]".to_string(),
                    400
                )
                    .into()
            )
        );

        // no return or NUL characters in values
        assert_eq!(
            normalize_and_validate(vec![(b"foo".to_vec(), b"bar\rbaz".to_vec())], false)
                .expect_err("Expect ProtocolError::LocalProtocolError"),
            ProtocolError::LocalProtocolError(
                (
                    "Illegal header value [98, 97, 114, 13, 98, 97, 122]".to_string(),
                    400
                )
                    .into()
            )
        );
        assert_eq!(
            normalize_and_validate(vec![(b"foo".to_vec(), b"bar\nbaz".to_vec())], false)
                .expect_err("Expect ProtocolError::LocalProtocolError"),
            ProtocolError::LocalProtocolError(
                (
                    "Illegal header value [98, 97, 114, 10, 98, 97, 122]".to_string(),
                    400
                )
                    .into()
            )
        );
        assert_eq!(
            normalize_and_validate(vec![(b"foo".to_vec(), b"bar\x00baz".to_vec())], false)
                .expect_err("Expect ProtocolError::LocalProtocolError"),
            ProtocolError::LocalProtocolError(
                (
                    "Illegal header value [98, 97, 114, 0, 98, 97, 122]".to_string(),
                    400
                )
                    .into()
            )
        );
        // no leading/trailing whitespace
        assert_eq!(
            normalize_and_validate(vec![(b"foo".to_vec(), b"barbaz  ".to_vec())], false)
                .expect_err("Expect ProtocolError::LocalProtocolError"),
            ProtocolError::LocalProtocolError(
                (
                    "Illegal header value [98, 97, 114, 98, 97, 122, 32, 32]".to_string(),
                    400
                )
                    .into()
            )
        );
        assert_eq!(
            normalize_and_validate(vec![(b"foo".to_vec(), b"  barbaz".to_vec())], false)
                .expect_err("Expect ProtocolError::LocalProtocolError"),
            ProtocolError::LocalProtocolError(
                (
                    "Illegal header value [32, 32, 98, 97, 114, 98, 97, 122]".to_string(),
                    400
                )
                    .into()
            )
        );
        assert_eq!(
            normalize_and_validate(vec![(b"foo".to_vec(), b"barbaz\t".to_vec())], false)
                .expect_err("Expect ProtocolError::LocalProtocolError"),
            ProtocolError::LocalProtocolError(
                (
                    "Illegal header value [98, 97, 114, 98, 97, 122, 9]".to_string(),
                    400
                )
                    .into()
            )
        );
        assert_eq!(
            normalize_and_validate(vec![(b"foo".to_vec(), b"\tbarbaz".to_vec())], false)
                .expect_err("Expect ProtocolError::LocalProtocolError"),
            ProtocolError::LocalProtocolError(
                (
                    "Illegal header value [9, 98, 97, 114, 98, 97, 122]".to_string(),
                    400
                )
                    .into()
            )
        );

        // content-length
        assert_eq!(
            normalize_and_validate(vec![(b"Content-Length".to_vec(), b"1".to_vec())], false)
                .unwrap(),
            Headers(vec![(
                b"Content-Length".to_vec(),
                b"content-length".to_vec(),
                b"1".to_vec()
            )])
        );
        assert_eq!(
            normalize_and_validate(vec![(b"Content-Length".to_vec(), b"asdf".to_vec())], false)
                .expect_err("Expect ProtocolError::LocalProtocolError"),
            ProtocolError::LocalProtocolError(("bad Content-Length".to_string(), 400).into())
        );
        assert_eq!(
            normalize_and_validate(vec![(b"Content-Length".to_vec(), b"1x".to_vec())], false)
                .expect_err("Expect ProtocolError::LocalProtocolError"),
            ProtocolError::LocalProtocolError(("bad Content-Length".to_string(), 400).into())
        );
        assert_eq!(
            normalize_and_validate(
                vec![
                    (b"Content-Length".to_vec(), b"1".to_vec()),
                    (b"Content-Length".to_vec(), b"2".to_vec())
                ],
                false
            )
            .expect_err("Expect ProtocolError::LocalProtocolError"),
            ProtocolError::LocalProtocolError(
                ("conflicting Content-Length headers".to_string(), 400).into()
            )
        );
        assert_eq!(
            normalize_and_validate(
                vec![
                    (b"Content-Length".to_vec(), b"0".to_vec()),
                    (b"Content-Length".to_vec(), b"0".to_vec())
                ],
                false
            )
            .unwrap(),
            Headers(vec![(
                b"Content-Length".to_vec(),
                b"content-length".to_vec(),
                b"0".to_vec()
            )])
        );
        assert_eq!(
            normalize_and_validate(vec![(b"Content-Length".to_vec(), b"0 , 0".to_vec())], false)
                .unwrap(),
            Headers(vec![(
                b"Content-Length".to_vec(),
                b"content-length".to_vec(),
                b"0".to_vec()
            )])
        );
        assert_eq!(
            normalize_and_validate(
                vec![
                    (b"Content-Length".to_vec(), b"1".to_vec()),
                    (b"Content-Length".to_vec(), b"1".to_vec()),
                    (b"Content-Length".to_vec(), b"2".to_vec())
                ],
                false
            )
            .expect_err("Expect ProtocolError::LocalProtocolError"),
            ProtocolError::LocalProtocolError(
                ("conflicting Content-Length headers".to_string(), 400).into()
            )
        );
        assert_eq!(
            normalize_and_validate(
                vec![(b"Content-Length".to_vec(), b"1 , 1,2".to_vec())],
                false
            )
            .expect_err("Expect ProtocolError::LocalProtocolError"),
            ProtocolError::LocalProtocolError(
                ("conflicting Content-Length headers".to_string(), 400).into()
            )
        );

        // transfer-encoding
        assert_eq!(
            normalize_and_validate(
                vec![(b"Transfer-Encoding".to_vec(), b"chunked".to_vec())],
                false
            )
            .unwrap(),
            Headers(vec![(
                b"Transfer-Encoding".to_vec(),
                b"transfer-encoding".to_vec(),
                b"chunked".to_vec()
            )])
        );
        assert_eq!(
            normalize_and_validate(
                vec![(b"Transfer-Encoding".to_vec(), b"cHuNkEd".to_vec())],
                false
            )
            .unwrap(),
            Headers(vec![(
                b"Transfer-Encoding".to_vec(),
                b"transfer-encoding".to_vec(),
                b"chunked".to_vec()
            )])
        );
        assert_eq!(
            normalize_and_validate(
                vec![(b"Transfer-Encoding".to_vec(), b"gzip".to_vec())],
                false
            )
            .expect_err("Expect ProtocolError::LocalProtocolError"),
            ProtocolError::LocalProtocolError(
                (
                    "Only Transfer-Encoding: chunked is supported".to_string(),
                    501
                )
                    .into()
            )
        );
        assert_eq!(
            normalize_and_validate(
                vec![
                    (b"Transfer-Encoding".to_vec(), b"chunked".to_vec()),
                    (b"Transfer-Encoding".to_vec(), b"gzip".to_vec())
                ],
                false
            )
            .expect_err("Expect ProtocolError::LocalProtocolError"),
            ProtocolError::LocalProtocolError(
                ("multiple Transfer-Encoding headers".to_string(), 501).into()
            )
        );
    }

    #[test]
    fn test_get_set_comma_header() {
        let headers = normalize_and_validate(
            vec![
                (b"Connection".to_vec(), b"close".to_vec()),
                (b"whatever".to_vec(), b"something".to_vec()),
                (b"connectiON".to_vec(), b"fOo,, , BAR".to_vec()),
            ],
            false,
        )
        .unwrap();

        assert_eq!(
            get_comma_header(&headers, b"connection"),
            vec![b"close".to_vec(), b"foo".to_vec(), b"bar".to_vec()]
        );

        let headers =
            set_comma_header(&headers, b"newthing", vec![b"a".to_vec(), b"b".to_vec()]).unwrap();

        assert_eq!(
            headers,
            Headers(vec![
                (
                    b"connection".to_vec(),
                    b"connection".to_vec(),
                    b"close".to_vec()
                ),
                (
                    b"whatever".to_vec(),
                    b"whatever".to_vec(),
                    b"something".to_vec()
                ),
                (
                    b"connection".to_vec(),
                    b"connection".to_vec(),
                    b"fOo,, , BAR".to_vec()
                ),
                (b"newthing".to_vec(), b"newthing".to_vec(), b"a".to_vec()),
                (b"newthing".to_vec(), b"newthing".to_vec(), b"b".to_vec()),
            ])
        );

        let headers =
            set_comma_header(&headers, b"whatever", vec![b"different thing".to_vec()]).unwrap();

        assert_eq!(
            headers,
            Headers(vec![
                (
                    b"connection".to_vec(),
                    b"connection".to_vec(),
                    b"close".to_vec()
                ),
                (
                    b"connection".to_vec(),
                    b"connection".to_vec(),
                    b"fOo,, , BAR".to_vec()
                ),
                (b"newthing".to_vec(), b"newthing".to_vec(), b"a".to_vec()),
                (b"newthing".to_vec(), b"newthing".to_vec(), b"b".to_vec()),
                (
                    b"whatever".to_vec(),
                    b"whatever".to_vec(),
                    b"different thing".to_vec()
                ),
            ])
        );
    }

    #[test]
    fn test_has_100_continue() {
        assert!(has_expect_100_continue(&Request {
            method: b"GET".to_vec(),
            target: b"/".to_vec(),
            headers: normalize_and_validate(
                vec![
                    (b"Host".to_vec(), b"example.com".to_vec()),
                    (b"Expect".to_vec(), b"100-continue".to_vec())
                ],
                false
            )
            .unwrap(),
            http_version: b"1.1".to_vec(),
        }));
        assert!(!has_expect_100_continue(&Request {
            method: b"GET".to_vec(),
            target: b"/".to_vec(),
            headers: normalize_and_validate(
                vec![(b"Host".to_vec(), b"example.com".to_vec())],
                false
            )
            .unwrap(),
            http_version: b"1.1".to_vec(),
        }));
        // Case insensitive
        assert!(has_expect_100_continue(&Request {
            method: b"GET".to_vec(),
            target: b"/".to_vec(),
            headers: normalize_and_validate(
                vec![
                    (b"Host".to_vec(), b"example.com".to_vec()),
                    (b"Expect".to_vec(), b"100-Continue".to_vec())
                ],
                false
            )
            .unwrap(),
            http_version: b"1.1".to_vec(),
        }));
        // Doesn't work in HTTP/1.0
        assert!(!has_expect_100_continue(&Request {
            method: b"GET".to_vec(),
            target: b"/".to_vec(),
            headers: normalize_and_validate(
                vec![
                    (b"Host".to_vec(), b"example.com".to_vec()),
                    (b"Expect".to_vec(), b"100-continue".to_vec())
                ],
                false
            )
            .unwrap(),
            http_version: b"1.0".to_vec(),
        }));
    }
}
