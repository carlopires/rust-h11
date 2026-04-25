use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct LocalProtocolError {
    pub message: String,
    pub code: u16,
}

impl From<(String, u16)> for LocalProtocolError {
    fn from(value: (String, u16)) -> Self {
        LocalProtocolError {
            message: value.0,
            code: value.1,
        }
    }
}

impl From<(&str, u16)> for LocalProtocolError {
    fn from(value: (&str, u16)) -> Self {
        LocalProtocolError {
            message: value.0.to_string(),
            code: value.1,
        }
    }
}

impl From<String> for LocalProtocolError {
    fn from(value: String) -> Self {
        LocalProtocolError {
            message: value,
            code: 400,
        }
    }
}

impl From<&str> for LocalProtocolError {
    fn from(value: &str) -> Self {
        LocalProtocolError {
            message: value.to_string(),
            code: 400,
        }
    }
}

impl LocalProtocolError {
    pub(crate) fn _reraise_as_remote_protocol_error(self) -> RemoteProtocolError {
        RemoteProtocolError {
            message: self.message,
            code: self.code,
        }
    }
}

impl fmt::Display for LocalProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (status code {})", self.message, self.code)
    }
}

impl std::error::Error for LocalProtocolError {}

#[derive(Debug, PartialEq, Eq)]
pub struct RemoteProtocolError {
    pub message: String,
    pub code: u16,
}

impl From<(String, u16)> for RemoteProtocolError {
    fn from(value: (String, u16)) -> Self {
        RemoteProtocolError {
            message: value.0,
            code: value.1,
        }
    }
}

impl From<(&str, u16)> for RemoteProtocolError {
    fn from(value: (&str, u16)) -> Self {
        RemoteProtocolError {
            message: value.0.to_string(),
            code: value.1,
        }
    }
}

impl From<String> for RemoteProtocolError {
    fn from(value: String) -> Self {
        RemoteProtocolError {
            message: value,
            code: 400,
        }
    }
}

impl From<&str> for RemoteProtocolError {
    fn from(value: &str) -> Self {
        RemoteProtocolError {
            message: value.to_string(),
            code: 400,
        }
    }
}

impl fmt::Display for RemoteProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (status code {})", self.message, self.code)
    }
}

impl std::error::Error for RemoteProtocolError {}

#[derive(Debug, PartialEq, Eq)]
pub enum ProtocolError {
    LocalProtocolError(LocalProtocolError),
    RemoteProtocolError(RemoteProtocolError),
}

impl From<LocalProtocolError> for ProtocolError {
    fn from(value: LocalProtocolError) -> Self {
        ProtocolError::LocalProtocolError(value)
    }
}

impl From<RemoteProtocolError> for ProtocolError {
    fn from(value: RemoteProtocolError) -> Self {
        ProtocolError::RemoteProtocolError(value)
    }
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LocalProtocolError(error) => write!(f, "local protocol error: {}", error),
            Self::RemoteProtocolError(error) => write!(f, "remote protocol error: {}", error),
        }
    }
}

impl std::error::Error for ProtocolError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::LocalProtocolError(error) => Some(error),
            Self::RemoteProtocolError(error) => Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn protocol_errors_implement_display_and_error() {
        let local = LocalProtocolError::from(("bad request", 400));
        assert_eq!(local.to_string(), "bad request (status code 400)");

        let remote = RemoteProtocolError::from(("bad gateway", 502));
        assert_eq!(remote.to_string(), "bad gateway (status code 502)");

        let error = ProtocolError::from(LocalProtocolError::from(("invalid", 400)));
        assert_eq!(
            error.to_string(),
            "local protocol error: invalid (status code 400)"
        );
        assert!(error.source().is_some());
    }
}
