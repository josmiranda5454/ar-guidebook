use axum::http::{header::AUTHORIZATION, HeaderMap, StatusCode};

use crate::AppState;

pub fn authorize(headers: &HeaderMap, state: &AppState) -> Result<(), StatusCode> {
    let expected = format!("Bearer {}", state.admin_token);
    match headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
    {
        Some(value) if value == expected => Ok(()),
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
