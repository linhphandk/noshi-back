use crate::shared::errors::AppResult;
use crate::state::AppState;
use crate::waitlist::models::WaitlistResponse;
use axum::extract::State;
use axum::Json;
use serde::Deserialize;
use tracing::instrument;

#[derive(Deserialize)]
pub struct JoinWaitlistRequest {
    pub email: String,
}

impl std::fmt::Debug for JoinWaitlistRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JoinWaitlistRequest")
            .field("email", &self.email)
            .finish()
    }
}

#[instrument(skip(state, req))]
pub async fn join_waitlist(
    State(state): State<AppState>,
    Json(req): Json<JoinWaitlistRequest>,
) -> AppResult<Json<WaitlistResponse>> {
    let response = state.waitlist_service.join(req.email).await?;
    Ok(Json(response))
}
