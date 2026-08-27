use crate::shared::errors::AppResult;
use crate::state::AppState;
use crate::waitlist::models::{JoinWaitlistRequest, WaitlistResponse};
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use tracing::instrument;

#[utoipa::path(
    post,
    path = "/api/waitlist",
    request_body = JoinWaitlistRequest,
    responses(
        (status = 201, description = "Joined waitlist", body = WaitlistResponse),
        (status = 409, description = "Email already on waitlist"),
        (status = 400, description = "Invalid email"),
    ),
    tag = "waitlist"
)]
#[instrument(skip(state, req))]
pub async fn join_waitlist(
    State(state): State<AppState>,
    Json(req): Json<JoinWaitlistRequest>,
) -> AppResult<(StatusCode, Json<WaitlistResponse>)> {
    let response = state.waitlist_service.join(req.email).await?;
    Ok((StatusCode::CREATED, Json(response)))
}
