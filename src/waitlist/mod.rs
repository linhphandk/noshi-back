pub mod controller;
pub mod models;
pub mod ports;
pub mod repository;
pub mod service;

use crate::state::AppState;
use axum::routing::post;
use axum::Router;

pub fn waitlist_router(_state: &AppState) -> Router<AppState> {
    Router::new().route("/api/waitlist", post(controller::join_waitlist))
}
