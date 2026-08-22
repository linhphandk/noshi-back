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

#[utoipa::path(
    post,
    path = "/profile",
    request_body = CreateProfileRequest,
    responses(
        (status = 201, description = "Profile created", body = Profile),
        (status = 400, description = "Validation error"),
        (status = 409, description = "Profile already exists"),
    ),
    security(
        ("bearer" = [])
    ),
    tag = "profile"
)]
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

#[utoipa::path(
    get,
    path = "/profile",
    responses(
        (status = 200, description = "Profile found", body = Profile),
        (status = 404, description = "Profile not found"),
    ),
    security(
        ("bearer" = [])
    ),
    tag = "profile"
)]
#[instrument(skip(state, user))]
pub async fn get_profile(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> AppResult<Json<Profile>> {
    let user_id = parse_user_id(&user)?;
    let profile = state.profile_service.get_profile(user_id).await?;
    Ok(Json(profile))
}

#[utoipa::path(
    put,
    path = "/profile",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated", body = Profile),
        (status = 404, description = "Profile not found"),
    ),
    security(
        ("bearer" = [])
    ),
    tag = "profile"
)]
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

#[utoipa::path(
    delete,
    path = "/profile",
    responses(
        (status = 200, description = "Profile deleted"),
        (status = 404, description = "Profile not found"),
    ),
    security(
        ("bearer" = [])
    ),
    tag = "profile"
)]
#[instrument(skip(state, user))]
pub async fn delete_profile(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> AppResult<StatusCode> {
    let user_id = parse_user_id(&user)?;
    state.profile_service.delete_profile(user_id).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    post,
    path = "/profile/platforms",
    request_body = CreateManualPlatformRequest,
    responses(
        (status = 201, description = "Platform added", body = ManualPlatform),
        (status = 404, description = "Profile not found"),
    ),
    security(
        ("bearer" = [])
    ),
    tag = "profile"
)]
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

#[utoipa::path(
    get,
    path = "/profile/platforms",
    responses(
        (status = 200, description = "List of platforms", body = Vec<ManualPlatform>),
    ),
    security(
        ("bearer" = [])
    ),
    tag = "profile"
)]
#[instrument(skip(state, user))]
pub async fn get_manual_platforms(
    Extension(user): Extension<AuthenticatedUser>,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<ManualPlatform>>> {
    let user_id = parse_user_id(&user)?;
    let platforms = state.profile_service.get_manual_platforms(user_id).await?;
    Ok(Json(platforms))
}

#[utoipa::path(
    put,
    path = "/profile/platforms/{platform_id}",
    params(
        ("platform_id" = Uuid, Path, description = "Platform ID"),
    ),
    request_body = UpdateManualPlatformRequest,
    responses(
        (status = 200, description = "Platform updated", body = ManualPlatform),
        (status = 404, description = "Platform not found"),
    ),
    security(
        ("bearer" = [])
    ),
    tag = "profile"
)]
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

#[utoipa::path(
    delete,
    path = "/profile/platforms/{platform_id}",
    params(
        ("platform_id" = Uuid, Path, description = "Platform ID"),
    ),
    responses(
        (status = 200, description = "Platform deleted"),
        (status = 404, description = "Platform not found"),
    ),
    security(
        ("bearer" = [])
    ),
    tag = "profile"
)]
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
