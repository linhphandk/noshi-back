use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use tracing::{debug, warn};

use crate::shared::types::AuthenticatedUser;
use crate::state::AppState;

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let token_owned = match request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
    {
        Some(h) if h.starts_with("Bearer ") => h[7..].to_string(),
        _ => {
            debug!("auth: missing or invalid Authorization header");
            return (
                StatusCode::UNAUTHORIZED,
                "Missing or invalid Authorization header",
            )
                .into_response();
        }
    };

    let user = match state
        .auth_service
        .get_current_user(token_owned.clone())
        .await
    {
        Ok(u) => {
            debug!(user_id = %u.id, email = %u.email, "auth: token valid");
            u
        }
        Err(_) => {
            warn!("auth: invalid or expired token");
            return (StatusCode::UNAUTHORIZED, "Invalid or expired token").into_response();
        }
    };

    request.extensions_mut().insert(AuthenticatedUser {
        id: user.id.to_string(),
        email: user.email,
        name: user.name,
        token: token_owned,
    });

    next.run(request).await
}
