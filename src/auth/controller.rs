use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use tracing::instrument;
use utoipa::ToSchema;

use crate::auth::models::{
    ForgotPasswordRequest, LoginRequest, LogoutRequest, RefreshRequest, RegisterRequest,
    ResetPasswordRequest,
};
use crate::shared::errors::AppResult;
use crate::shared::types::AuthenticatedUser;
use crate::state::AppState;

#[derive(Serialize, ToSchema)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
}

#[instrument(skip(state, req), fields(email = %req.email))]
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<(StatusCode, Json<AuthResponse>)> {
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
        refresh_token: tokens.refresh_token,
    };
    Ok((StatusCode::CREATED, Json(response)))
}

#[instrument(skip(state, req), fields(email = %req.email))]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<(StatusCode, Json<AuthResponse>)> {
    let (user, tokens) = state.auth_service.login(req.email, req.password).await?;
    let response = AuthResponse {
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            name: user.name,
        },
        access_token: tokens.access_token,
        expires_in: tokens.expires_in,
        refresh_token: tokens.refresh_token,
    };
    Ok((StatusCode::OK, Json(response)))
}

#[instrument(skip(state, req))]
pub async fn logout(
    State(state): State<AppState>,
    Json(req): Json<LogoutRequest>,
) -> AppResult<StatusCode> {
    state.auth_service.logout(req.refresh_token).await?;
    Ok(StatusCode::OK)
}

#[instrument(skip(state, req))]
pub async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> AppResult<(StatusCode, Json<AuthResponse>)> {
    let (user, tokens) = state.auth_service.refresh_token(req.refresh_token).await?;
    let response = AuthResponse {
        user: UserResponse {
            id: user.id.to_string(),
            email: user.email,
            name: user.name,
        },
        access_token: tokens.access_token,
        expires_in: tokens.expires_in,
        refresh_token: tokens.refresh_token,
    };
    Ok((StatusCode::OK, Json(response)))
}

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

#[instrument(skip(state, req))]
pub async fn forgot_password(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> AppResult<StatusCode> {
    state.auth_service.forgot_password(req.email).await?;
    Ok(StatusCode::OK)
}

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
