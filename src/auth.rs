use axum::{
    Json,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Serialize;
use std::sync::OnceLock;

static PASSWORD: OnceLock<Option<String>> = OnceLock::new();

pub fn init_password(password: Option<String>) {
    PASSWORD
        .set(password)
        .expect("Password already initialized");
}

pub fn is_auth_required() -> bool {
    PASSWORD.get().map(|p| p.is_some()).unwrap_or(false)
}

fn get_password() -> Option<&'static String> {
    PASSWORD.get().and_then(|p| p.as_ref())
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthStatusResponse {
    auth_required: bool,
}

pub async fn auth_status_handler() -> Json<AuthStatusResponse> {
    Json(AuthStatusResponse {
        auth_required: is_auth_required(),
    })
}

pub async fn auth_middleware(request: Request, next: Next) -> Response {
    let Some(expected_password) = get_password() else {
        return next.run(request).await;
    };

    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    let Some(auth_value) = auth_header else {
        return (StatusCode::UNAUTHORIZED, "Missing Authorization header").into_response();
    };

    let Some(encoded) = auth_value.strip_prefix("Bearer ") else {
        return (StatusCode::UNAUTHORIZED, "Invalid Authorization format").into_response();
    };

    let Ok(decoded_bytes) = STANDARD.decode(encoded) else {
        return (StatusCode::UNAUTHORIZED, "Invalid base64 encoding").into_response();
    };

    let Ok(provided_password) = String::from_utf8(decoded_bytes) else {
        return (StatusCode::UNAUTHORIZED, "Invalid UTF-8 in password").into_response();
    };

    if &provided_password != expected_password {
        return (StatusCode::UNAUTHORIZED, "Invalid password").into_response();
    }

    next.run(request).await
}
