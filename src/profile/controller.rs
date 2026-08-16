use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::Json;
use tracing::instrument;

use crate::profile::models::{
    CreateManualPlatformRequest, CreateProfileRequest, ManualPlatform, Profile,
    UpdateManualPlatformRequest, UpdateProfileRequest,
};
use crate::shared::errors::{AppError, AppResult};
use crate::shared::types::AuthenticatedUser;
use crate::state::AppState;

fn parse_user_id(user: &AuthenticatedUser) -> AppResult<uuid::Uuid> {
    user.id
        .parse()
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))
}

#[instrument(skip(state, user, req))]
pub async fn create_profile(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(req): Json<CreateProfileRequest>,
) -> AppResult<(StatusCode, Json<Profile>)> {
    let user_id = parse_user_id(&user)?;
    let profile = state.profile_service.create_profile(user_id, req).await?;
    Ok((StatusCode::CREATED, Json(profile)))
}

#[instrument(skip(state, user))]
pub async fn get_profile(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> AppResult<Json<Profile>> {
    let user_id = parse_user_id(&user)?;
    let profile = state.profile_service.get_profile(user_id).await?;
    Ok(Json(profile))
}

#[instrument(skip(state, user, req))]
pub async fn update_profile(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(req): Json<UpdateProfileRequest>,
) -> AppResult<Json<Profile>> {
    let user_id = parse_user_id(&user)?;
    let profile = state.profile_service.update_profile(user_id, req).await?;
    Ok(Json(profile))
}

#[instrument(skip(state, user))]
pub async fn delete_profile(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> AppResult<StatusCode> {
    let user_id = parse_user_id(&user)?;
    state.profile_service.delete_profile(user_id).await?;
    Ok(StatusCode::OK)
}

#[instrument(skip(state, user, req))]
pub async fn add_manual_platform(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Json(req): Json<CreateManualPlatformRequest>,
) -> AppResult<(StatusCode, Json<ManualPlatform>)> {
    let user_id = parse_user_id(&user)?;
    let platform = state
        .profile_service
        .add_manual_platform(user_id, req)
        .await?;
    Ok((StatusCode::CREATED, Json(platform)))
}

#[instrument(skip(state, user))]
pub async fn get_manual_platforms(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<ManualPlatform>>> {
    let user_id = parse_user_id(&user)?;
    let platforms = state.profile_service.get_manual_platforms(user_id).await?;
    Ok(Json(platforms))
}

#[instrument(skip(state, user, platform_id, req))]
pub async fn update_manual_platform(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(platform_id): Path<uuid::Uuid>,
    Json(req): Json<UpdateManualPlatformRequest>,
) -> AppResult<Json<ManualPlatform>> {
    let user_id = parse_user_id(&user)?;
    let platform = state
        .profile_service
        .update_manual_platform(user_id, platform_id, req)
        .await?;
    Ok(Json(platform))
}

#[instrument(skip(state, user, platform_id))]
pub async fn delete_manual_platform(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
    Path(platform_id): Path<uuid::Uuid>,
) -> AppResult<StatusCode> {
    let user_id = parse_user_id(&user)?;
    state
        .profile_service
        .delete_manual_platform(user_id, platform_id)
        .await?;
    Ok(StatusCode::OK)
}
