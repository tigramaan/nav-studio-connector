use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fmt::{Display, Formatter};

pub type Result<T> = std::result::Result<T, ConnectorError>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    DiscoveryTimeout,
    InvalidServiceRecord,
    InvalidUrl,
    EndpointUnreachable,
    TlsFetchFailed,
    IdentityReceiptInvalid,
    IdentityMismatch,
    HumanConfirmationRequired,
    PermissionDenied,
    TrustStoreFailed,
    HealthCheckFailed,
    UnsupportedPlatform,
    InvalidInput,
    InternalError,
}

impl ErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DiscoveryTimeout => "DISCOVERY_TIMEOUT",
            Self::InvalidServiceRecord => "INVALID_SERVICE_RECORD",
            Self::InvalidUrl => "INVALID_URL",
            Self::EndpointUnreachable => "ENDPOINT_UNREACHABLE",
            Self::TlsFetchFailed => "TLS_FETCH_FAILED",
            Self::IdentityReceiptInvalid => "IDENTITY_RECEIPT_INVALID",
            Self::IdentityMismatch => "IDENTITY_MISMATCH",
            Self::HumanConfirmationRequired => "HUMAN_CONFIRMATION_REQUIRED",
            Self::PermissionDenied => "PERMISSION_DENIED",
            Self::TrustStoreFailed => "TRUST_STORE_FAILED",
            Self::HealthCheckFailed => "HEALTH_CHECK_FAILED",
            Self::UnsupportedPlatform => "UNSUPPORTED_PLATFORM",
            Self::InvalidInput => "INVALID_INPUT",
            Self::InternalError => "INTERNAL_ERROR",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorError {
    pub code: ErrorCode,
    pub message: String,
    pub retryable: bool,
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub details: Map<String, Value>,
}

impl ConnectorError {
    pub fn new(code: ErrorCode, message: impl Into<String>, retryable: bool) -> Self {
        Self {
            code,
            message: message.into(),
            retryable,
            details: Map::new(),
        }
    }

    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<Value>) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }

    pub fn exit_code(&self) -> i32 {
        match self.code {
            ErrorCode::InvalidInput | ErrorCode::InvalidUrl | ErrorCode::InvalidServiceRecord => 2,
            ErrorCode::IdentityMismatch
            | ErrorCode::IdentityReceiptInvalid
            | ErrorCode::HumanConfirmationRequired => 3,
            ErrorCode::DiscoveryTimeout
            | ErrorCode::EndpointUnreachable
            | ErrorCode::TlsFetchFailed
            | ErrorCode::HealthCheckFailed => 4,
            ErrorCode::PermissionDenied
            | ErrorCode::TrustStoreFailed
            | ErrorCode::UnsupportedPlatform => 5,
            ErrorCode::InternalError => 1,
        }
    }
}

impl Display for ConnectorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for ConnectorError {}
