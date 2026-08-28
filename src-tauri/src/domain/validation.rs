use super::{ConnectorError, ErrorCode, Result};
use url::Url;

pub fn validate_https_url(value: &str) -> Result<Url> {
    if value.len() > 2048 {
        return Err(ConnectorError::new(
            ErrorCode::InvalidUrl,
            "URL exceeds 2048 characters",
            false,
        ));
    }
    let parsed = Url::parse(value).map_err(|_| {
        ConnectorError::new(
            ErrorCode::InvalidUrl,
            "Expected an absolute HTTPS URL",
            false,
        )
    })?;
    if parsed.scheme() != "https" {
        return Err(ConnectorError::new(
            ErrorCode::InvalidUrl,
            "Only https:// endpoints are allowed",
            false,
        ));
    }
    if parsed.host_str().is_none() || !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(ConnectorError::new(
            ErrorCode::InvalidUrl,
            "URL host is missing or credentials are embedded",
            false,
        ));
    }
    if parsed.fragment().is_some() {
        return Err(ConnectorError::new(
            ErrorCode::InvalidUrl,
            "URL fragments are not accepted",
            false,
        ));
    }
    Ok(parsed)
}

pub fn normalize_fingerprint(value: &str) -> Result<String> {
    let normalized: String = value
        .chars()
        .filter(|ch| *ch != ':' && *ch != '-' && !ch.is_ascii_whitespace())
        .map(|ch| ch.to_ascii_uppercase())
        .collect();
    if normalized.len() != 64 || !normalized.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(ConnectorError::new(
            ErrorCode::InvalidInput,
            "SHA-256 fingerprint must contain exactly 64 hexadecimal characters",
            false,
        ));
    }
    Ok(normalized)
}

pub fn validate_timeout(seconds: u64) -> Result<u64> {
    if !(1..=30).contains(&seconds) {
        return Err(ConnectorError::new(
            ErrorCode::InvalidInput,
            "Timeout must be between 1 and 30 seconds",
            false,
        ));
    }
    Ok(seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_https_without_credentials() {
        assert_eq!(
            validate_https_url("https://robot.local:8780/")
                .unwrap()
                .host_str(),
            Some("robot.local")
        );
    }

    #[test]
    fn rejects_non_https_and_credentials() {
        assert!(validate_https_url("http://robot.local").is_err());
        assert!(validate_https_url("https://user:pass@robot.local").is_err());
    }

    #[test]
    fn normalizes_sha256() {
        let raw = "AA:".repeat(31) + "AA";
        assert_eq!(normalize_fingerprint(&raw).unwrap(), "AA".repeat(32));
        assert!(normalize_fingerprint("abc").is_err());
    }
}
