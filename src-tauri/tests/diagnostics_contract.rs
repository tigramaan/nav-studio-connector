use nav_studio_connector_lib::application::diagnose;

#[test]
fn diagnostics_are_bounded_and_secret_free() {
    let report = diagnose(None, 1);
    let json = serde_json::to_string(&report).unwrap().to_ascii_lowercase();
    assert!(json.len() < 64 * 1024);
    assert!(!json.contains("private key"));
    assert!(!json.contains("password"));
    assert!(!json.contains("access_token"));
}
