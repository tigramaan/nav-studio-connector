mod error;
mod identity;
mod model;
mod validation;

pub use error::{ConnectorError, ErrorCode, Result};
pub use identity::{authorize_trust, AuthorizationMethod, CallerMode, TrustAuthorization};
pub use model::{
    ConnectorStatus, DiagnosticCheck, DiagnosticReport, DiscoveredRobot, EndpointInspection,
    HealthState, IdentityState, TrustOperation, TrustState,
};
pub use validation::{normalize_fingerprint, validate_https_url, validate_timeout};
