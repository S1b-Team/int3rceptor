use std::sync::Arc;

use crate::state::AppState;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    if let Some(token) = &state.api_token {
        let bearer = format!("Bearer {}", token);
        let authorized = req
            .headers()
            .get("Authorization")
            .and_then(|value| value.to_str().ok())
            .map(|value| value == bearer)
            .unwrap_or(false)
            || req
                .headers()
                .get("x-api-key")
                .and_then(|value| value.to_str().ok())
                .map(|value| value == token.as_str())
                .unwrap_or(false);
        if !authorized {
            tracing::warn!("auth_failed" = true, path = %req.uri());
            return StatusCode::UNAUTHORIZED.into_response();
        }
    }
    next.run(req).await
}
