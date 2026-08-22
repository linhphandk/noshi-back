pub mod controller;
pub mod models;
pub mod ports;
pub mod repository;
pub mod service;

use crate::shared::middleware::auth_middleware;
use crate::state::AppState;
use axum::routing::{delete, get, post, put};

pub fn profile_router(state: &AppState) -> axum::Router<AppState> {
    let public = axum::Router::<AppState>::new().route(
        "/profile/public/{slug}",
        get(controller::get_public_profile),
    );

    let protected = axum::Router::<AppState>::new()
        .route("/profile", get(controller::get_profile))
        .route("/profile", post(controller::create_profile))
        .route("/profile", put(controller::update_profile))
        .route("/profile", delete(controller::delete_profile))
        .route("/profile/platforms", get(controller::get_manual_platforms))
        .route("/profile/platforms", post(controller::add_manual_platform))
        .route(
            "/profile/platforms/{platform_id}",
            put(controller::update_manual_platform),
        )
        .route(
            "/profile/platforms/{platform_id}",
            delete(controller::delete_manual_platform),
        )
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    axum::Router::<AppState>::new()
        .merge(public)
        .merge(protected)
}
