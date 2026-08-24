pub mod controller;
pub mod instagram;
pub mod models;
pub mod ports;
pub mod repository;
pub mod service;

use axum::routing::{delete, get, post};
use axum::Router;

use crate::shared::middleware::auth_middleware;
use crate::state::AppState;

pub fn social_router(state: &AppState) -> Router<AppState> {
    let protected = Router::<AppState>::new()
        .route(
            "/social/{platform}/authorize",
            get(crate::social::controller::get_authorize_url),
        )
        .route("/social/connect", post(crate::social::controller::connect))
        .route(
            "/social/connections",
            get(crate::social::controller::list_connections),
        )
        .route(
            "/social/connections/{id}",
            delete(crate::social::controller::disconnect),
        )
        .route(
            "/social/connections/{id}/sync",
            post(crate::social::controller::sync),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    Router::<AppState>::new().merge(protected)
}
