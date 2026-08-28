use nav_studio_connector_lib::domain::{authorize_trust, CallerMode, ErrorCode};

#[test]
fn unattended_unknown_identity_fails_closed() {
    let observed = "A".repeat(64);
    let error = authorize_trust(&observed, None, false, false, CallerMode::Agent).unwrap_err();
    assert_eq!(error.code, ErrorCode::HumanConfirmationRequired);
}

#[test]
fn independently_expected_fingerprint_must_match_exactly() {
    let observed = "A".repeat(64);
    let expected = "B".repeat(64);
    let error =
        authorize_trust(&observed, Some(&expected), false, false, CallerMode::Agent).unwrap_err();
    assert_eq!(error.code, ErrorCode::IdentityMismatch);
}
