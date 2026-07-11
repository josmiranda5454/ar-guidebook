use axum::http::{header::AUTHORIZATION, HeaderMap, StatusCode};
use subtle::ConstantTimeEq;

use crate::AppState;

pub fn authorize(headers: &HeaderMap, state: &AppState) -> Result<(), StatusCode> {
    authorize_token(headers, &state.admin_token)
}

pub fn authorize_recorder(headers: &HeaderMap, state: &AppState) -> Result<(), StatusCode> {
    authorize_token(headers, &state.recorder_token)
}

fn authorize_token(headers: &HeaderMap, expected_token: &str) -> Result<(), StatusCode> {
    let expected = format!("Bearer {expected_token}");
    match headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    {
        Some(value) if value.as_bytes().ct_eq(expected.as_bytes()).into() => Ok(()),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

pub fn configured_admin_credentials() -> (String, String, String) {
    (
        std::env::var("CLIMBAR_ADMIN_EMAIL").unwrap_or_else(|_| "admin@example.com".into()),
        std::env::var("CLIMBAR_ADMIN_PASSWORD").unwrap_or_else(|_| "dev-password".into()),
        std::env::var("CLIMBAR_ADMIN_TOKEN").unwrap_or_else(|_| "dev-admin-token".into()),
    )
}

pub fn configured_recorder_token() -> String {
    std::env::var("CLIMBAR_RECORDER_TOKEN").unwrap_or_else(|_| "dev-recorder-token".into())
}

pub fn validate_production_configuration() {
    if std::env::var("CLIMBAR_ENV").as_deref() != Ok("production") {
        return;
    }

    let (email, password, admin_token) = configured_admin_credentials();
    let recorder_token = configured_recorder_token();
    let allowed_origins = std::env::var("CLIMBAR_ALLOWED_ORIGINS").unwrap_or_default();

    assert!(
        email != "admin@example.com",
        "CLIMBAR_ADMIN_EMAIL must be changed in production"
    );
    assert!(
        password.len() >= 16,
        "CLIMBAR_ADMIN_PASSWORD must be at least 16 characters"
    );
    assert!(
        admin_token.len() >= 32,
        "CLIMBAR_ADMIN_TOKEN must be at least 32 characters"
    );
    assert!(
        recorder_token.len() >= 32,
        "CLIMBAR_RECORDER_TOKEN must be at least 32 characters"
    );
    assert!(
        !allowed_origins.trim().is_empty(),
        "CLIMBAR_ALLOWED_ORIGINS is required in production"
    );
    assert!(
        admin_token != "dev-admin-token",
        "default admin token is not allowed in production"
    );
    assert!(
        recorder_token != "dev-recorder-token",
        "default recorder token is not allowed in production"
    );
}

#[cfg(test)]
mod tests {
    use super::authorize_token;
    use axum::http::{header::AUTHORIZATION, HeaderMap, HeaderValue};

    #[test]
    fn authorization_requires_an_exact_bearer_token() {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer recorder"));

        assert!(authorize_token(&headers, "recorder").is_ok());
        assert!(authorize_token(&headers, "recorders").is_err());
        assert!(authorize_token(&headers, "Recorder").is_err());
    }
}
