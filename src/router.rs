use axum::routing::get;
use axum::Router;

use crate::auth::auth_router;
use crate::state::AppState;
use crate::waitlist::waitlist_router;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .merge(waitlist_router(&state))
        .merge(auth_router(&state))
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}
