use axum::extract::{Extension, State};
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::Serialize;
use tracing::instrument;
use utoipa::ToSchema;

use crate::auth::models::{
    ForgotPasswordRequest, LoginRequest, RegisterRequest, ResetPasswordRequest,
};
use crate::shared::errors::AppResult;
use crate::shared::types::AuthenticatedUser;
use crate::state::AppState;

const REFRESH_COOKIE_NAME: &str = "refresh_token";
const REFRESH_COOKIE_MAX_AGE: &str = "604800"; // 7 days

fn refresh_cookie(value: &str) -> String {
    format!(
        "{name}={value}; HttpOnly; SameSite=Strict; Path=/auth; Max-Age={max_age}",
        name = REFRESH_COOKIE_NAME,
        value = value,
        max_age = REFRESH_COOKIE_MAX_AGE,
    )
}

fn clear_refresh_cookie() -> String {
    format!(
        "{name}=; HttpOnly; SameSite=Strict; Path=/auth; Max-Age=0",
        name = REFRESH_COOKIE_NAME,
    )
}

fn extract_refresh_token(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get("cookie")?.to_str().ok()?;
    for part in cookie_header.split(';') {
        let mut kv = part.trim().splitn(2, '=');
        let key = kv.next()?.trim();
        let val = kv.next()?.trim();
        if key == REFRESH_COOKIE_NAME {
            return Some(val.to_string());
        }
    }
    None
}

#[derive(Serialize, ToSchema)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub expires_in: u64,
}

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
}

#[utoipa::path(
    post,
    path = "/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered", body = AuthResponse),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Email already registered"),
    ),
    tag = "auth"
)]
#[instrument(skip(state, req), fields(email = %req.email))]
pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RegisterRequest>,
) -> AppResult<(StatusCode, HeaderMap, Json<AuthResponse>)> {
    let (user, tokens) = state
        .auth_service
        .register(req.email, req.name, req.password)
        .await?;
    let response = AuthResponse {
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            name: user.name,
        },
        access_token: tokens.access_token,
        expires_in: tokens.expires_in,
    };
    let mut resp_headers = HeaderMap::new();
    if let Some(rt) = &tokens.refresh_token {
        resp_headers.insert(SET_COOKIE, refresh_cookie(rt).parse().unwrap());
    }
    Ok((StatusCode::CREATED, resp_headers, Json(response)))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Logged in", body = AuthResponse),
        (status = 400, description = "Invalid credentials"),
    ),
    tag = "auth"
)]
#[instrument(skip(state, req), fields(email = %req.email))]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<(StatusCode, HeaderMap, Json<AuthResponse>)> {
    let (user, tokens) = state.auth_service.login(req.email, req.password).await?;
    let response = AuthResponse {
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            name: user.name,
        },
        access_token: tokens.access_token,
        expires_in: tokens.expires_in,
    };
    let mut resp_headers = HeaderMap::new();
    if let Some(rt) = &tokens.refresh_token {
        resp_headers.insert(SET_COOKIE, refresh_cookie(rt).parse().unwrap());
    }
    Ok((StatusCode::OK, resp_headers, Json(response)))
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    responses(
        (status = 200, description = "Logged out"),
    ),
    tag = "auth"
)]
#[instrument(skip(state, headers))]
pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<(StatusCode, HeaderMap)> {
    if let Some(rt) = extract_refresh_token(&headers) {
        let _ = state.auth_service.logout(rt).await;
    }
    let mut resp_headers = HeaderMap::new();
    resp_headers.insert(SET_COOKIE, clear_refresh_cookie().parse().unwrap());
    Ok((StatusCode::OK, resp_headers))
}

#[utoipa::path(
    post,
    path = "/auth/refresh",
    responses(
        (status = 200, description = "Token refreshed", body = AuthResponse),
        (status = 401, description = "Missing or invalid refresh token"),
    ),
    tag = "auth"
)]
#[instrument(skip(state, headers))]
pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<(StatusCode, HeaderMap, Json<AuthResponse>)> {
    let rt = extract_refresh_token(&headers).ok_or_else(|| {
        crate::shared::errors::AppError::Unauthorized("Missing refresh token cookie".to_string())
    })?;
    let (user, tokens) = state.auth_service.refresh_token(rt).await?;
    let response = AuthResponse {
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            name: user.name,
        },
        access_token: tokens.access_token,
        expires_in: tokens.expires_in,
    };
    let mut resp_headers = HeaderMap::new();
    if let Some(rt) = &tokens.refresh_token {
        resp_headers.insert(SET_COOKIE, refresh_cookie(rt).parse().unwrap());
    }
    Ok((StatusCode::OK, resp_headers, Json(response)))
}

#[utoipa::path(
    get,
    path = "/auth/me",
    responses(
        (status = 200, description = "Current user", body = UserResponse),
        (status = 401, description = "Unauthorized"),
    ),
    security(
        ("bearer" = [])
    ),
    tag = "auth"
)]
#[instrument(skip(state))]
pub async fn me(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> AppResult<Json<UserResponse>> {
    let user = state.auth_service.get_current_user(user.token).await?;
    Ok(Json(UserResponse {
        id: user.id.to_string(),
        email: user.email,
        name: user.name,
    }))
}

#[utoipa::path(
    post,
    path = "/auth/forgot-password",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Reset email sent"),
    ),
    tag = "auth"
)]
#[instrument(skip(state, req))]
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> AppResult<StatusCode> {
    state.auth_service.forgot_password(req.email).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/auth/reset-password",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset"),
        (status = 400, description = "Invalid or expired token"),
    ),
    tag = "auth"
)]
#[instrument(skip(state, req))]
pub async fn reset_password(
    State(state): State<AppState>,
    Json(req): Json<ResetPasswordRequest>,
) -> AppResult<StatusCode> {
    state
        .auth_service
        .reset_password(req.token, req.password)
        .await?;
    Ok(StatusCode::OK)
}
