use super::{normalize_fingerprint, ConnectorError, ErrorCode, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CallerMode {
    Human,
    Agent,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationMethod {
    SignedReceipt,
    HumanFingerprint,
    ExpectedFingerprint,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrustAuthorization {
    pub method: AuthorizationMethod,
    pub fingerprint_sha256: String,
}

pub fn authorize_trust(
    observed: &str,
    expected: Option<&str>,
    receipt_verified: bool,
    human_confirmed: bool,
    caller_mode: CallerMode,
) -> Result<TrustAuthorization> {
    let observed = normalize_fingerprint(observed)?;
    if receipt_verified {
        return Ok(TrustAuthorization {
            method: AuthorizationMethod::SignedReceipt,
            fingerprint_sha256: observed,
        });
    }

    if let Some(expected) = expected {
        let expected = normalize_fingerprint(expected)?;
        if expected != observed {
            return Err(ConnectorError::new(
                ErrorCode::IdentityMismatch,
                "Observed certificate fingerprint does not match the expected fingerprint",
                false,
            )
            .with_detail("expected", expected)
            .with_detail("observed", observed));
        }
        let method = if caller_mode == CallerMode::Human && human_confirmed {
            AuthorizationMethod::HumanFingerprint
        } else {
            AuthorizationMethod::ExpectedFingerprint
        };
        return Ok(TrustAuthorization {
            method,
            fingerprint_sha256: observed,
        });
    }

    if caller_mode == CallerMode::Human && human_confirmed {
        return Err(ConnectorError::new(
            ErrorCode::HumanConfirmationRequired,
            "Human confirmation must include the independently displayed expected fingerprint",
            false,
        ));
    }

    Err(ConnectorError::new(
        ErrorCode::HumanConfirmationRequired,
        "Unknown certificate identity requires signed receipt verification or an independently supplied SHA-256 fingerprint",
        false,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fp(ch: char) -> String {
        std::iter::repeat(ch).take(64).collect()
    }

    #[test]
    fn rejects_mismatch() {
        let error =
            authorize_trust(&fp('A'), Some(&fp('B')), false, false, CallerMode::Agent).unwrap_err();
        assert_eq!(error.code, ErrorCode::IdentityMismatch);
    }

    #[test]
    fn agent_requires_expected_or_receipt() {
        let error = authorize_trust(&fp('A'), None, false, false, CallerMode::Agent).unwrap_err();
        assert_eq!(error.code, ErrorCode::HumanConfirmationRequired);
    }

    #[test]
    fn records_human_authorization() {
        let fp = fp('A');
        let result = authorize_trust(&fp, Some(&fp), false, true, CallerMode::Human).unwrap();
        assert_eq!(result.method, AuthorizationMethod::HumanFingerprint);
    }
}
