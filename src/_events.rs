use crate::_abnf::{METHOD, REASON_PHRASE, REQUEST_TARGET};
use crate::{_headers::Headers, _util::ProtocolError};
use lazy_static::lazy_static;
use regex::bytes::Regex;
use std::fmt::{self, Formatter};

lazy_static! {
    static ref HTTP_VERSION_RE: Regex = Regex::new(r"^[0-9]\.[0-9]$").unwrap();
    static ref METHOD_RE: Regex = Regex::new(&format!(r"^{}$", *METHOD)).unwrap();
    static ref REASON_RE: Regex = Regex::new(&format!(r"^{}$", *REASON_PHRASE)).unwrap();
    static ref REQUEST_TARGET_RE: Regex = Regex::new(&format!(r"^{}$", *REQUEST_TARGET)).unwrap();
}

#[derive(Clone, PartialEq, Eq, Default)]
pub struct Request {
    pub method: Vec<u8>,
    pub headers: Headers,
    pub target: Vec<u8>,
    pub http_version: Vec<u8>,
}

impl Request {
    pub fn new<M, T, V>(
        method: M,
        headers: Headers,
        target: T,
        http_version: V,
    ) -> Result<Self, ProtocolError>
    where
        M: AsRef<[u8]>,
        T: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        let request = Self {
            method: method.as_ref().to_vec(),
            headers,
            target: target.as_ref().to_vec(),
            http_version: http_version.as_ref().to_vec(),
        };
        request.validate()?;
        Ok(request)
    }

    pub fn new_http11<M, T>(method: M, headers: Headers, target: T) -> Result<Self, ProtocolError>
    where
        M: AsRef<[u8]>,
        T: AsRef<[u8]>,
    {
        Self::new(method, headers, target, b"1.1")
    }

    pub fn validate(&self) -> Result<(), ProtocolError> {
        let mut host_count = 0;
        for (name, _) in self.headers.iter() {
            if name == b"host" {
                host_count += 1;
            }
        }
        if !HTTP_VERSION_RE.is_match(&self.http_version) {
            return Err(ProtocolError::LocalProtocolError(
                ("Illegal HTTP version".to_string(), 400).into(),
            ));
        }
        if self.http_version == b"1.1" && host_count == 0 {
            return Err(ProtocolError::LocalProtocolError(
                ("Missing mandatory Host: header".to_string(), 400).into(),
            ));
        }
        if host_count > 1 {
            return Err(ProtocolError::LocalProtocolError(
                ("Found multiple Host: headers".to_string(), 400).into(),
            ));
        }

        if !METHOD_RE.is_match(&self.method) {
            return Err(ProtocolError::LocalProtocolError(
                ("Illegal method characters".to_string(), 400).into(),
            ));
        }
        if !REQUEST_TARGET_RE.is_match(&self.target) {
            return Err(ProtocolError::LocalProtocolError(
                ("Illegal target characters".to_string(), 400).into(),
            ));
        }

        Ok(())
    }
}

impl std::fmt::Debug for Request {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Request")
            .field("method", &String::from_utf8_lossy(&self.method))
            .field("headers", &self.headers)
            .field("target", &String::from_utf8_lossy(&self.target))
            .field("http_version", &String::from_utf8_lossy(&self.http_version))
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Response {
    pub headers: Headers,
    pub http_version: Vec<u8>,
    pub reason: Vec<u8>,
    pub status_code: u16,
}

impl Response {
    pub fn new<R, V>(
        status_code: u16,
        headers: Headers,
        reason: R,
        http_version: V,
    ) -> Result<Self, ProtocolError>
    where
        R: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        let response = Self {
            headers,
            http_version: http_version.as_ref().to_vec(),
            reason: reason.as_ref().to_vec(),
            status_code,
        };
        response.validate()?;
        Ok(response)
    }

    pub fn new_http11<R>(
        status_code: u16,
        headers: Headers,
        reason: R,
    ) -> Result<Self, ProtocolError>
    where
        R: AsRef<[u8]>,
    {
        Self::new(status_code, headers, reason, b"1.1")
    }

    pub fn validate(&self) -> Result<(), ProtocolError> {
        if !(100..=999).contains(&self.status_code) {
            return Err(ProtocolError::LocalProtocolError(
                ("Illegal status code".to_string(), 400).into(),
            ));
        }
        if !HTTP_VERSION_RE.is_match(&self.http_version) {
            return Err(ProtocolError::LocalProtocolError(
                ("Illegal HTTP version".to_string(), 400).into(),
            ));
        }
        if !REASON_RE.is_match(&self.reason) {
            return Err(ProtocolError::LocalProtocolError(
                ("Illegal reason phrase".to_string(), 400).into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Data {
    pub data: Vec<u8>,
    pub chunk_start: bool,
    pub chunk_end: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EndOfMessage {
    pub headers: Headers,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConnectionClosed {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Request(Request),
    NormalResponse(Response),
    InformationalResponse(Response),
    Data(Data),
    EndOfMessage(EndOfMessage),
    ConnectionClosed(ConnectionClosed),
    NeedData(),
    Paused(),
}

impl From<Request> for Event {
    fn from(request: Request) -> Self {
        Self::Request(request)
    }
}

impl From<Response> for Event {
    fn from(response: Response) -> Self {
        match response.status_code {
            100..=199 => Self::InformationalResponse(response),
            _ => Self::NormalResponse(response),
        }
    }
}

impl From<Data> for Event {
    fn from(data: Data) -> Self {
        Self::Data(data)
    }
}

impl From<EndOfMessage> for Event {
    fn from(end_of_message: EndOfMessage) -> Self {
        Self::EndOfMessage(end_of_message)
    }
}

impl From<ConnectionClosed> for Event {
    fn from(connection_closed: ConnectionClosed) -> Self {
        Self::ConnectionClosed(connection_closed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_new_rejects_invalid_input() {
        assert!(Response::new(99, Headers::default(), b"OK".to_vec(), b"1.1".to_vec()).is_err());
        assert!(Response::new(1000, Headers::default(), b"OK".to_vec(), b"1.1".to_vec()).is_err());
        assert!(Response::new(
            200,
            Headers::default(),
            b"OK".to_vec(),
            b"HTTP/1.1".to_vec()
        )
        .is_err());
        assert!(Response::new(
            200,
            Headers::default(),
            b"bad\nreason".to_vec(),
            b"1.1".to_vec()
        )
        .is_err());
    }

    #[test]
    fn test_request_new_rejects_invalid_http_version() {
        assert!(Request::new(
            b"GET".to_vec(),
            Headers::new(vec![(b"Host".to_vec(), b"example.com".to_vec())]).unwrap(),
            b"/".to_vec(),
            b"HTTP/1.1".to_vec(),
        )
        .is_err());
    }

    #[test]
    fn test_request_new_accepts_borrowed_inputs_and_http11_default() {
        let request =
            Request::new_http11("GET", Headers::new([("Host", "example.com")]).unwrap(), "/")
                .unwrap();

        assert_eq!(request.method, b"GET");
        assert_eq!(request.target, b"/");
        assert_eq!(request.http_version, b"1.1");
    }

    #[test]
    fn test_response_new_accepts_borrowed_inputs_and_http11_default() {
        let response =
            Response::new_http11(200, Headers::new([("Content-Length", "0")]).unwrap(), "OK")
                .unwrap();

        assert_eq!(response.status_code, 200);
        assert_eq!(response.reason, b"OK");
        assert_eq!(response.http_version, b"1.1");
    }
}
