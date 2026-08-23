use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::Json;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::shared::errors::{AppError, AppResult};
use crate::shared::types::AuthenticatedUser;
use crate::social::models::{AuthorizeUrlResponse, ConnectSocialRequest, SocialConnectionResponse};
use crate::state::AppState;

#[derive(Serialize, Deserialize)]
struct StateClaims {
    sub: String,
    typ: String,
    exp: usize,
}

fn generate_state_token(user_id: Uuid, state_secret: &str) -> String {
    let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::minutes(10))
        .unwrap()
        .timestamp() as usize;

    let claims = StateClaims {
        sub: user_id.to_string(),
        typ: "social_oauth".to_string(),
        exp,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state_secret.as_bytes()),
    )
    .unwrap()
}

fn verify_state_token(state: &str, state_secret: &str) -> Result<Uuid, AppError> {
    let token_data = decode::<StateClaims>(
        state,
        &DecodingKey::from_secret(state_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid state token".to_string()))?;

    if token_data.claims.typ != "social_oauth" {
        return Err(AppError::Unauthorized(
            "Invalid state token type".to_string(),
        ));
    }

    Uuid::parse_str(&token_data.claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid state token subject".to_string()))
}

#[utoipa::path(
    get,
    path = "/social/{platform}/authorize",
    params(
        ("platform" = String, Path, description = "Platform name (instagram, tiktok, youtube)"),
    ),
    responses(
        (status = 200, description = "Authorization URL", body = AuthorizeUrlResponse),
        (status = 400, description = "Unsupported platform"),
    ),
    security(
        ("bearer" = [])
    ),
    tag = "social"
)]
#[instrument(skip(state))]
pub async fn get_authorize_url(
    State(state): State<AppState>,
    Path(platform): Path<String>,
    Extension(user): Extension<AuthenticatedUser>,
) -> AppResult<Json<AuthorizeUrlResponse>> {
    let provider = state
        .social_service
        .providers
        .get(&platform)
        .ok_or_else(|| AppError::BadRequest(format!("Unsupported platform: {}", platform)))?;

    let user_id: Uuid = user
        .id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    let state_token = generate_state_token(user_id, &state.social_oauth_state_secret);
    let authorize_url = provider.authorize_url(&state_token);

    Ok(Json(AuthorizeUrlResponse { authorize_url }))
}

#[utoipa::path(
    post,
    path = "/social/connect",
    request_body = ConnectSocialRequest,
    responses(
        (status = 200, description = "Connected", body = SocialConnectionResponse),
        (status = 400, description = "Invalid request or unsupported platform"),
        (status = 401, description = "Invalid state token"),
    ),
    security(
        ("bearer" = [])
    ),
    tag = "social"
)]
#[instrument(skip(state, req))]
pub async fn connect(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<ConnectSocialRequest>,
) -> AppResult<Json<SocialConnectionResponse>> {
    let user_id: Uuid = user
        .id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    verify_state_token(&req.state, &state.social_oauth_state_secret)?;

    let connection = state.social_service.connect(user_id, req).await?;
    Ok(Json(connection))
}

#[utoipa::path(
    get,
    path = "/social/connections",
    responses(
        (status = 200, description = "List of connections", body = Vec<SocialConnectionResponse>),
    ),
    security(
        ("bearer" = [])
    ),
    tag = "social"
)]
#[instrument(skip(state, user))]
pub async fn list_connections(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> AppResult<Json<Vec<SocialConnectionResponse>>> {
    let user_id: Uuid = user
        .id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    let connections = state.social_service.list_connections(user_id).await?;
    Ok(Json(connections))
}

#[utoipa::path(
    delete,
    path = "/social/connections/{id}",
    params(
        ("id" = Uuid, Path, description = "Connection ID"),
    ),
    responses(
        (status = 200, description = "Disconnected"),
        (status = 404, description = "Connection not found"),
        (status = 403, description = "Not your connection"),
    ),
    security(
        ("bearer" = [])
    ),
    tag = "social"
)]
#[instrument(skip(state, user))]
pub async fn disconnect(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<uuid::Uuid>,
) -> AppResult<StatusCode> {
    let user_id: Uuid = user
        .id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    state.social_service.disconnect(user_id, id).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/social/connections/{id}/sync",
    params(
        ("id" = Uuid, Path, description = "Connection ID"),
    ),
    responses(
        (status = 200, description = "Synced", body = SocialConnectionResponse),
        (status = 404, description = "Connection not found"),
        (status = 403, description = "Not your connection"),
    ),
    security(
        ("bearer" = [])
    ),
    tag = "social"
)]
#[instrument(skip(state, user))]
pub async fn sync(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<uuid::Uuid>,
) -> AppResult<Json<SocialConnectionResponse>> {
    let user_id: Uuid = user
        .id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    let connection = state.social_service.sync(user_id, id).await?;
    Ok(Json(connection))
}
