use axum::http::{header::AUTHORIZATION, HeaderMap, StatusCode};
use std::{
    collections::HashMap,
    sync::RwLock,
    time::{Duration, Instant},
};

use subtle::ConstantTimeEq;
use uuid::Uuid;

use crate::AppState;

pub fn authorize(headers: &HeaderMap, state: &AppState) -> Result<(), StatusCode> {
    authorize_token(headers, &state.admin_token)
}

pub fn authorize_recorder(headers: &HeaderMap, state: &AppState) -> Result<(), StatusCode> {
    let Some(token) = bearer_token(headers) else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let now = Instant::now();
    let mut sessions = state
        .recorder_sessions
        .write()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    sessions.retain(|_, expires_at| *expires_at > now);

    if sessions
        .keys()
        .any(|expected| token.as_bytes().ct_eq(expected.as_bytes()).into())
    {
        Ok(())
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

fn authorize_token(headers: &HeaderMap, expected_token: &str) -> Result<(), StatusCode> {
    match bearer_token(headers) {
        Some(value) if value.as_bytes().ct_eq(expected_token.as_bytes()).into() => Ok(()),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
}

pub fn configured_admin_credentials() -> (String, String, String) {
    (
        std::env::var("CLIMBAR_ADMIN_EMAIL").unwrap_or_else(|_| "admin@example.com".into()),
        std::env::var("CLIMBAR_ADMIN_PASSWORD").unwrap_or_else(|_| "dev-password".into()),
        std::env::var("CLIMBAR_ADMIN_TOKEN").unwrap_or_else(|_| "dev-admin-token".into()),
    )
}

pub fn configured_recorder_credentials() -> (String, String) {
    (
        std::env::var("CLIMBAR_RECORDER_EMAIL").unwrap_or_else(|_| "recorder@example.com".into()),
        std::env::var("CLIMBAR_RECORDER_PASSWORD")
            .unwrap_or_else(|_| "dev-recorder-password".into()),
    )
}

pub fn issue_recorder_session(
    sessions: &RwLock<HashMap<String, Instant>>,
) -> Result<(String, Instant), StatusCode> {
    let token = Uuid::new_v4().to_string();
    let expires_at = Instant::now() + Duration::from_secs(8 * 60 * 60);
    sessions
        .write()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .insert(token.clone(), expires_at);
    Ok((token, expires_at))
}

pub fn validate_production_configuration() {
    if std::env::var("CLIMBAR_ENV").as_deref() != Ok("production") {
        return;
    }

    let (email, password, admin_token) = configured_admin_credentials();
    let (recorder_email, recorder_password) = configured_recorder_credentials();
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
        recorder_email.contains('@'),
        "CLIMBAR_RECORDER_EMAIL must be valid in production"
    );
    assert!(
        recorder_password.len() >= 16,
        "CLIMBAR_RECORDER_PASSWORD must be at least 16 characters"
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
        recorder_password != "dev-recorder-password",
        "default recorder password is not allowed in production"
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
